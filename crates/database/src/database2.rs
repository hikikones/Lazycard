use std::path::Path;

use crate::{scheduler2::*, sqlite::*};

pub struct Database {
    sqlite: Sqlite,
    scheduler: Scheduler,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> SqliteResult<Self> {
        let db = Self {
            sqlite: Sqlite::open(path)?,
            scheduler: Scheduler::new(),
        };
        db.migrate();
        Ok(db)
    }

    pub fn open_in_memory() -> SqliteResult<Self> {
        let db = Self {
            sqlite: Sqlite::open_in_memory()?,
            scheduler: Scheduler::new(),
        };
        db.migrate();
        Ok(db)
    }

    pub fn path(&self) -> Option<&Path> {
        self.sqlite.path().map(Path::new)
    }

    pub fn add_card(&self, content: &str) -> SqliteResult<CardId> {
        self.sqlite
            .execute_with_args("INSERT INTO cards (content) VALUES (?)", [content])?;
        Ok(CardId(self.sqlite.last_insert_rowid()))
    }

    pub fn get_cards(&self, buf: &mut Vec<CardId>) -> SqliteResult<()> {
        self.sqlite.query("SELECT id FROM cards", |row| {
            let id = row.get(0)?;
            buf.push(id);
            Ok(())
        })
    }

    pub fn get_card_content(&self, id: CardId, f: impl FnOnce(&str)) -> SqliteResult<()> {
        self.sqlite.query_single_with_args(
            "SELECT id, content FROM cards WHERE id = ?",
            [id],
            |row| {
                let content = row.get_ref(1)?.as_str()?;
                f(content);
                Ok(())
            },
        )
    }

    pub fn get_due_count(&self) -> SqliteResult<u32> {
        self.sqlite.query_single(
            "SELECT COUNT(id) FROM cards WHERE due_time <= (unixepoch('now'))",
            |row| row.get(0),
        )
    }

    pub fn get_due_cards(&self, buf: &mut Vec<CardId>) -> SqliteResult<()> {
        self.sqlite.query(
            "SELECT id FROM cards WHERE due_time <= (unixepoch('now'))",
            |row| {
                let id = row.get(0)?;
                buf.push(id);
                Ok(())
            },
        )
    }

    pub fn get_due_card_random(&self) -> SqliteResult<Option<CardId>> {
        self.get_due_card_random_except(CardId::ZERO)
    }

    pub fn get_due_card_random_except(&self, id: CardId) -> SqliteResult<Option<CardId>> {
        self.sqlite.query_first_with_args(
            "
            SELECT id, due_time FROM cards \
            WHERE due_time <= (unixepoch('now')) AND id != ? \
            ORDER BY RANDOM() \
            LIMIT 1
            ",
            [id],
            |row| row.get(0),
        )
    }

    pub fn update_card(&self, id: CardId, content: &str) -> SqliteResult<()> {
        self.sqlite
            .execute_with_args("UPDATE cards SET content = ?1 WHERE id = ?2", (content, id))?;
        Ok(())
    }

    pub fn delete_card(&self, id: CardId) -> SqliteResult<()> {
        self.sqlite
            .execute_with_args("DELETE FROM cards WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn review_card(&self, id: CardId, success: bool) -> SqliteResult<ReviewId> {
        let (create_time, stability, difficulty) = self.sqlite.query_single_with_args(
            "SELECT create_time, stability, difficulty FROM cards WHERE id = ?",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        let last_review_time = self.sqlite.query_first_with_args(
            "SELECT time FROM reviews \
                WHERE card_id = ? \
                ORDER BY time DESC \
                LIMIT 1",
            [id],
            |row| row.get(0),
        )?;

        let current_state = CurrentReviewState {
            stability,
            difficulty,
            last_review_time: last_review_time.unwrap_or(create_time),
        };
        let next_state = self.scheduler.schedule(current_state, success);
        self.sqlite.execute_with_args(
            "UPDATE cards \
            SET stability = ?1, difficulty = ?2, due_time = ?3 \
            WHERE id = ?4",
            (
                next_state.stability,
                next_state.difficulty,
                next_state.due_time,
                id,
            ),
        )?;
        self.sqlite.execute_with_args(
            "INSERT INTO reviews (success, card_id) VALUES (?1, ?2)",
            (success, id),
        )?;

        Ok(ReviewId(self.sqlite.last_insert_rowid()))
    }

    pub fn search(&self, input: &str, buf: &mut Vec<CardId>) -> SqliteResult<()> {
        self.sqlite.query_with_args(
            "
            SELECT rowid FROM cards_fts \
            WHERE cards_fts MATCH ? \
            ORDER BY rank",
            [input],
            |row| {
                let id = row.get(0)?;
                buf.push(id);
                Ok(())
            },
        )
    }

    pub fn search_highlight(&self, id: CardId, input: &str, buf: &mut String) -> SqliteResult<()> {
        self.sqlite.query_single_with_args(
            "
            SELECT rowid, highlight(cards_fts, 0, '<b>', '</b>') FROM cards_fts \
            WHERE rowid = ?1 AND cards_fts MATCH ?2",
            (id, input),
            |row| {
                let highlighted_content = row.get_ref(1)?.as_str()?;
                buf.push_str(highlighted_content);
                Ok(())
            },
        )
    }

    fn migrate(&self) {
        const VERSION: SqliteId = 1;

        match self.sqlite.version() {
            0 => {
                self.sqlite
                    .execute_batch(include_str!("schema_v1.sql"))
                    .unwrap();
                self.sqlite.set_version(VERSION);
                add_test_data(self);
            }
            VERSION => {}
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardId(SqliteId);

impl CardId {
    pub const ZERO: Self = Self(0);
}

impl ToSql for CardId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Integer(self.0)))
    }
}

impl FromSql for CardId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_i64().map(|n| Self(n))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReviewId(SqliteId);

impl ToSql for ReviewId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Integer(self.0)))
    }
}

impl FromSql for ReviewId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_i64().map(|n| Self(n))
    }
}

fn add_test_data(db: &Database) {
    db.add_card(
            r#"
# this is a comment
*left* paragraph with *bold*, _italic_ and maybe `verbatim text` that *should wrap* when line becomes _*tooooooooo*_ long..*.*

- item 1
- item 2 with *lots of text* that can wrap to next line but also keeping the indent so it looks nice ohhh yeah
- item 3

> right paragraph

| center paragraph

![ image description text ]( assets/wallpaper.jpg )

```rust
fn main() {
    println!("Hello, world!");
}
```

---

![  ]( assets/meow.png )

Lorem ipsum dolor sit amet,
consectetur adipiscing elit.
Donec fermentum ipsum nec sagittis feugiat.
Curabitur pulvinar et orci luctus faucibus.
In erat justo, placerat et risus quis, cursus elementum mi.

![ image description text ]( assets/tall2.jpg )

---

another one

---

and another one

![ image description text ]( assets/tall.gif )
"#,
    ).unwrap();

    db.add_card(
            r#"
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum ipsum nec sagittis feugiat. Curabitur pulvinar et orci luctus faucibus. In erat justo, placerat et risus quis, cursus elementum mi. Donec non leo est. Etiam consectetur, lectus nec auctor sodales, velit arcu tincidunt justo, mattis dictum odio libero ac enim. Morbi maximus, tortor id ornare tristique, purus mi pulvinar urna, vel accumsan ante nunc vel urna. Maecenas ligula elit, tempor eget augue ut, auctor blandit ligula. Mauris maximus condimentum aliquam. Nam purus enim, ornare ut suscipit et, bibendum a ex. Donec euismod velit quis nisi convallis, rhoncus lacinia sapien aliquam. Ut dolor magna, imperdiet eu arcu vitae, consectetur feugiat velit. Donec eu dolor eu tortor rutrum egestas quis in orci. Maecenas pulvinar, massa eget fringilla convallis, dolor tellus luctus elit, in tempus nibh dolor at dolor. Nulla viverra et justo sit amet rhoncus. Fusce porttitor odio in lectus ullamcorper vehicula.

Nullam pharetra augue non leo maximus convallis a sit amet risus. Integer congue laoreet efficitur. Interdum et malesuada fames ac ante ipsum primis in faucibus. Quisque vestibulum, nisl vitae porttitor maximus, nulla mauris cursus ligula, vel consectetur magna dolor ac nulla. Integer at pharetra nisi, quis congue ante. Nam et varius purus. Maecenas placerat sapien at ante cursus, at volutpat felis facilisis. Fusce id metus vel urna molestie tincidunt. Phasellus sed dapibus ligula.

Vivamus nec dui pellentesque, interdum dui vitae, egestas ex. Morbi malesuada porta vehicula. Donec dapibus mattis arcu, vel bibendum diam lobortis et. Curabitur sit amet felis aliquet enim mollis semper. Mauris diam orci, rutrum sed neque vel, malesuada sollicitudin augue. Integer vehicula dolor consectetur tincidunt imperdiet. Donec at dui urna. Duis et turpis in diam ornare volutpat.

Mauris suscipit imperdiet mi et semper. Nam nec lorem sagittis, lobortis ligula convallis, hendrerit tortor. Morbi dapibus magna ut sollicitudin placerat. Etiam sodales varius ante convallis hendrerit. Duis varius quam et tincidunt vestibulum. Fusce facilisis elit eu ligula consequat, dignissim elementum diam aliquet. Maecenas ipsum tellus, condimentum sit amet ultrices et, semper id nibh. Fusce varius porttitor dui vitae fermentum. Nam placerat, nunc eget tempus egestas, enim nisl scelerisque erat, in venenatis leo leo a leo. Nam quis nibh vitae turpis efficitur lobortis id a metus.

In aliquet dui sapien, ut semper elit sodales sed. Proin quis libero luctus libero scelerisque ornare eget id mi. Cras accumsan arcu ut ante pharetra fringilla. Sed feugiat placerat dolor, et feugiat lorem iaculis id. Cras interdum est nec elit molestie, sed aliquam quam hendrerit. Proin sit amet pellentesque enim. Nam bibendum, mauris vel eleifend ultricies, orci sapien hendrerit sem, a tristique lectus nulla a ligula.
"#,
    ).unwrap();

    db.add_card("👻 oijwqwu qwdiowhq  i hio h qiowhqwheqw👻👻 wwq qiuwhdidwh👻👻👻❤️\n\nauhui ❤️awudhia\n🧑‍🌾❤️👨‍🦰jfpkw huiw wjwioj ijf weoijwioejfiowejfiowjfiowej\n\nthis\tis\ta\tparagraph\twith\ttabs\n\n```rust\nfn main() {\n\tprintln!(\"Hello, world!\");\n}\n```").unwrap();
}
