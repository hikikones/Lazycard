use std::path::Path;

use utils::sqlite::*;

mod scheduler;

pub use scheduler::*;

pub type DatabaseResult<T> = Result<T, DatabaseError>;

// TODO: Add tests.

pub struct Database {
    sqlite: Sqlite,
    scheduler: Scheduler,
    s: String,
}

impl Database {
    fn new(sqlite: Sqlite) -> Self {
        Self {
            sqlite,
            scheduler: Scheduler::new(),
            s: String::new(),
        }
    }

    pub fn open(path: impl AsRef<Path>) -> DatabaseResult<Self> {
        let db = Self::new(Sqlite::open(path)?);
        db.migrate()?;
        Ok(db)
    }

    pub fn open_in_memory() -> DatabaseResult<Self> {
        let db = Self::new(Sqlite::open_in_memory()?);
        db.migrate()?;
        Ok(db)
    }

    pub fn path(&self) -> Option<&Path> {
        self.sqlite.path().map(Path::new)
    }

    pub fn is_cards_empty(&self) -> bool {
        self.sqlite
            .query_single("SELECT NOT EXISTS (SELECT 1 FROM cards)", (), |row| {
                row.get(0)
            })
            .unwrap()
    }

    pub fn add_card(&self, content: &str) -> CardId {
        self.sqlite
            .execute("INSERT INTO cards (content) VALUES (?)", [content])
            .unwrap();
        CardId(self.sqlite.last_insert_rowid())
    }

    pub fn get_cards(&self, mut f: impl FnMut(CardId)) {
        self.sqlite
            .query(
                "SELECT id FROM cards ORDER BY create_time DESC",
                (),
                |row| Ok(f(row.get(0)?)),
            )
            .unwrap()
    }

    pub fn get_cards_with_tags(
        &mut self,
        includes: impl ExactSizeIterator<Item = TagId>,
        excludes: impl ExactSizeIterator<Item = TagId>,
        mut f: impl FnMut(CardId),
    ) {
        self.s.clear();
        let mut itoa = itoa::Buffer::new();

        // All cards
        self.s.push_str("SELECT c.id FROM cards c");

        // Filter by include tags
        let next_token = if includes.len() > 0 {
            let len = includes.len();
            self.s
                .push_str(" JOIN card_tags ct ON ct.card_id = c.id WHERE ct.tag_id IN (");
            for tid in includes {
                self.s.extend([itoa.format(tid.0), ","]);
            }
            self.s.pop();
            self.s
                .push_str(") GROUP BY c.id HAVING COUNT(DISTINCT ct.tag_id) = ");
            self.s.push_str(itoa.format(len));
            " AND "
        } else {
            " WHERE "
        };

        // Filter by exclude tags
        if excludes.len() > 0 {
            self.s.push_str(next_token);
            self.s.push_str(
                "NOT EXISTS (SELECT 1 FROM card_tags ct2 WHERE ct2.card_id = c.id AND ct2.tag_id IN (",
            );
            for tid in excludes {
                self.s.extend([itoa.format(tid.0), ","]);
            }
            self.s.pop();
            self.s.push_str("))");
        }

        // Sort by newest
        self.s.push_str(" ORDER BY c.create_time DESC");

        self.sqlite
            .query(self.s.as_str(), (), |row| Ok(f(row.get(0)?)))
            .unwrap();
    }

    pub fn get_cards_without_tags(&self, mut f: impl FnMut(CardId)) {
        self.sqlite
            .query(
                "SELECT c.id FROM cards c WHERE NOT EXISTS (
                    SELECT 1 FROM card_tags ct \
                    WHERE ct.card_id = c.id \
                ) ORDER BY c.create_time DESC",
                (),
                |row| Ok(f(row.get(0)?)),
            )
            .unwrap();
    }

    pub fn get_card_content(&self, id: CardId, f: impl FnOnce(&str)) {
        self.sqlite
            .query_single("SELECT id, content FROM cards WHERE id = ?", [id], |row| {
                Ok(f(row.get_ref(1)?.as_str()?))
            })
            .unwrap();
    }

    pub fn get_due_count(&self) -> u32 {
        self.sqlite
            .query_single(
                "SELECT COUNT(id) FROM cards WHERE due_time <= (unixepoch('now'))",
                (),
                |row| row.get(0),
            )
            .unwrap()
    }

    pub fn get_due_cards(&self, mut f: impl FnMut(CardId)) {
        self.sqlite
            .query(
                "SELECT id FROM cards WHERE due_time <= (unixepoch('now'))",
                (),
                |row| Ok(f(row.get(0)?)),
            )
            .unwrap();
    }

    pub fn get_due_card_random(&self) -> Option<CardId> {
        self.get_due_card_random_except(CardId::ZERO)
    }

    pub fn get_due_card_random_except(&self, id: CardId) -> Option<CardId> {
        self.sqlite
            .query_first(
                "
            SELECT id, due_time FROM cards \
            WHERE due_time <= (unixepoch('now')) AND id != ? \
            ORDER BY RANDOM() \
            LIMIT 1
            ",
                [id],
                |row| row.get(0),
            )
            .unwrap()
    }

    pub fn search(&self, input: &str, mut f: impl FnMut(CardId)) {
        self.sqlite
            .query(
                "
            SELECT rowid FROM cards_fts \
            WHERE cards_fts MATCH ? \
            ORDER BY rank",
                [input],
                |row| Ok(f(row.get(0)?)),
            )
            .unwrap();
    }

    pub fn search_highlight(&self, id: CardId, input: &str, f: impl FnOnce(&str)) {
        const ANSI_REVERSE: &str = "\x1b[7m";
        const ANSI_NOT_REVERSE: &str = "\x1b[27m";

        self.sqlite
            .query_single(
                constcat::concat!(
                    "SELECT rowid, highlight(cards_fts, 0, '",
                    ANSI_REVERSE,
                    "', '",
                    ANSI_NOT_REVERSE,
                    "') FROM cards_fts ",
                    "WHERE rowid = ?1 AND cards_fts MATCH ?2"
                ),
                (id, input),
                |row| Ok(f(row.get_ref(1)?.as_str()?)),
            )
            .unwrap();
    }

    pub fn update_card(&self, id: CardId, content: &str) {
        self.sqlite
            .execute("UPDATE cards SET content = ?1 WHERE id = ?2", (content, id))
            .unwrap();
    }

    pub fn review_card(&self, id: CardId, success: bool, desired_retention: f32) -> ReviewId {
        let (create_time, stability, difficulty) = self
            .sqlite
            .query_single(
                "SELECT create_time, stability, difficulty FROM cards WHERE id = ?",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        let last_review_time = self
            .sqlite
            .query_first(
                "SELECT time FROM reviews \
                WHERE card_id = ? \
                ORDER BY time DESC \
                LIMIT 1",
                [id],
                |row| row.get(0),
            )
            .unwrap();

        let current_state = CurrentReviewState {
            stability,
            difficulty,
            last_review_time: last_review_time.unwrap_or(create_time),
        };
        let next_state = self
            .scheduler
            .schedule(current_state, success, desired_retention);
        self.sqlite
            .execute(
                "UPDATE cards \
            SET stability = ?1, difficulty = ?2, due_time = ?3 \
            WHERE id = ?4",
                (
                    next_state.stability,
                    next_state.difficulty,
                    next_state.due_time,
                    id,
                ),
            )
            .unwrap();
        self.sqlite
            .execute(
                "INSERT INTO reviews (success, card_id) VALUES (?1, ?2)",
                (success, id),
            )
            .unwrap();

        ReviewId(self.sqlite.last_insert_rowid())
    }

    pub fn delete_card(&self, id: CardId) {
        self.sqlite
            .execute("DELETE FROM cards WHERE id = ?", [id])
            .unwrap();
    }

    pub fn add_tag(&self, name: &str) -> Option<TagId> {
        if self
            .sqlite
            .query_first("SELECT id FROM tags WHERE name = ?", [name], |row| {
                row.get::<_, TagId>(0)
            })
            .unwrap()
            .is_some()
        {
            return None;
        }

        self.sqlite
            .execute("INSERT INTO tags (name) VALUES (?)", [name])
            .unwrap();
        Some(TagId(self.sqlite.last_insert_rowid()))
    }

    pub fn get_tags(&self, mut f: impl FnMut(TagId)) {
        self.sqlite
            .query("SELECT id FROM tags ORDER BY name", (), |row| {
                Ok(f(row.get(0)?))
            })
            .unwrap();
    }

    pub fn get_tags_and_name(&self, mut f: impl FnMut(TagId, &str)) {
        self.sqlite
            .query("SELECT id, name FROM tags", (), |row| {
                Ok(f(row.get(0)?, row.get_ref(1)?.as_str()?))
            })
            .unwrap();
    }

    pub fn get_tags_and_cards_count(&self, mut f: impl FnMut(TagId, &str, u32)) {
        self.sqlite
            .query(
                "SELECT t.id, t.name, COUNT(ct.card_id) FROM tags t \
                    LEFT JOIN card_tags ct ON ct.tag_id = t.id \
                    GROUP BY t.id, t.name \
                    ORDER BY t.name",
                (),
                |row| Ok(f(row.get(0)?, row.get_ref(1)?.as_str()?, row.get(2)?)),
            )
            .unwrap();
    }

    pub fn get_tag_name(&self, id: TagId, f: impl FnOnce(&str)) {
        self.sqlite
            .query_single("SELECT id, name FROM tags WHERE id = ?", [id], |row| {
                Ok(f(row.get_ref(1)?.as_str()?))
            })
            .unwrap();
    }

    pub fn update_tag(&self, id: TagId, name: &str) -> bool {
        if self
            .sqlite
            .query_first("SELECT id FROM tags WHERE name = ?", [name], |row| {
                row.get::<_, TagId>(0)
            })
            .unwrap()
            .is_some()
        {
            return false;
        }

        self.sqlite
            .execute("UPDATE tags SET name = ?1 WHERE id = ?2", (name, id))
            .unwrap();
        true
    }

    pub fn delete_tag(&self, id: TagId) {
        self.sqlite
            .execute("DELETE FROM tags WHERE id = ?", [id])
            .unwrap();
    }

    pub fn add_tag_for_card(&self, cid: CardId, tid: TagId) {
        self.sqlite
            .execute(
                "INSERT INTO card_tags (card_id, tag_id) VALUES (?1, ?2)",
                (cid, tid),
            )
            .unwrap();
    }

    pub fn get_tags_for_card(&self, id: CardId, mut f: impl FnMut(TagId)) {
        self.sqlite
            .query(
                "SELECT tag_id FROM card_tags WHERE card_id = ?",
                [id],
                |row| Ok(f(row.get(0)?)),
            )
            .unwrap();
    }

    pub fn get_cards_count_for_tag(&self, id: TagId) -> u32 {
        self.sqlite
            .query_single(
                "SELECT COUNT(*) FROM card_tags WHERE tag_id = ?",
                [id],
                |row| row.get(0),
            )
            .unwrap()
    }

    pub fn get_name_and_cards_count_for_tag(&self, id: TagId, f: impl FnOnce(&str, u32)) {
        self.sqlite
            .query_single(
                "SELECT t.name, COUNT(ct.card_id) FROM tags t \
                    LEFT JOIN card_tags ct ON ct.tag_id = t.id \
                    WHERE t.id = ? \
                    GROUP BY t.id, t.name",
                [id],
                |row| Ok(f(row.get_ref(0)?.as_str()?, row.get(1)?)),
            )
            .unwrap()
    }

    pub fn delete_tag_for_card(&self, cid: CardId, tid: TagId) {
        self.sqlite
            .execute(
                "DELETE FROM card_tags WHERE card_id = ?1 AND tag_id = ?2",
                (cid, tid),
            )
            .unwrap();
    }

    pub fn delete_cards_with_tag(&self, tid: TagId) {
        self.sqlite
            .execute(
                "DELETE FROM cards WHERE id IN ( \
                        SELECT card_id FROM card_tags WHERE tag_id = ? \
                    )",
                [tid],
            )
            .unwrap();
    }

    fn migrate(&self) -> DatabaseResult<()> {
        const VERSION: SqliteId = 1;

        let version = self.sqlite.version();
        match version {
            0 => {
                self.sqlite
                    .execute_batch(include_str!("database/schema_v1.sql"))?;
                self.sqlite.set_version(VERSION);
                add_test_data(self);
            }
            VERSION => {}
            _ => return DatabaseResult::Err(DatabaseError::Version(version)),
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardId(SqliteId);

impl CardId {
    const ZERO: Self = Self(0);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TagId(SqliteId);

impl ToSql for TagId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Integer(self.0)))
    }
}

impl FromSql for TagId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_i64().map(|n| Self(n))
    }
}

#[derive(Debug)]
pub enum DatabaseError {
    Sqlite(SqliteError),
    Version(SqliteId),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::Sqlite(err) => err.fmt(f),
            &DatabaseError::Version(v) => f.write_fmt(format_args!("unknown version: {v}")),
        }
    }
}

impl std::error::Error for DatabaseError {}

impl From<SqliteError> for DatabaseError {
    fn from(err: SqliteError) -> Self {
        Self::Sqlite(err)
    }
}

fn add_test_data(db: &Database) {
    let cid1= db.add_card(
            r#"
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum ipsum nec sagittis feugiat. Curabitur pulvinar et orci luctus faucibus. In erat justo, placerat et risus quis, cursus elementum mi. Donec non leo est. Etiam consectetur, lectus nec auctor sodales, velit arcu tincidunt justo, mattis dictum odio libero ac enim. Morbi maximus, tortor id ornare tristique, purus mi pulvinar urna, vel accumsan ante nunc vel urna. Maecenas ligula elit, tempor eget augue ut, auctor blandit ligula. Mauris maximus condimentum aliquam. Nam purus enim, ornare ut suscipit et, bibendum a ex. Donec euismod velit quis nisi convallis, rhoncus lacinia sapien aliquam. Ut dolor magna, imperdiet eu arcu vitae, consectetur feugiat velit. Donec eu dolor eu tortor rutrum egestas quis in orci. Maecenas pulvinar, massa eget fringilla convallis, dolor tellus luctus elit, in tempus nibh dolor at dolor. Nulla viverra et justo sit amet rhoncus. Fusce porttitor odio in lectus ullamcorper vehicula.

Nullam pharetra augue non leo maximus convallis a sit amet risus. Integer congue laoreet efficitur. Interdum et malesuada fames ac ante ipsum primis in faucibus. Quisque vestibulum, nisl vitae porttitor maximus, nulla mauris cursus ligula, vel consectetur magna dolor ac nulla. Integer at pharetra nisi, quis congue ante. Nam et varius purus. Maecenas placerat sapien at ante cursus, at volutpat felis facilisis. Fusce id metus vel urna molestie tincidunt. Phasellus sed dapibus ligula.

Vivamus nec dui pellentesque, interdum dui vitae, egestas ex. Morbi malesuada porta vehicula. Donec dapibus mattis arcu, vel bibendum diam lobortis et. Curabitur sit amet felis aliquet enim mollis semper. Mauris diam orci, rutrum sed neque vel, malesuada sollicitudin augue. Integer vehicula dolor consectetur tincidunt imperdiet. Donec at dui urna. Duis et turpis in diam ornare volutpat.

Mauris suscipit imperdiet mi et semper. Nam nec lorem sagittis, lobortis ligula convallis, hendrerit tortor. Morbi dapibus magna ut sollicitudin placerat. Etiam sodales varius ante convallis hendrerit. Duis varius quam et tincidunt vestibulum. Fusce facilisis elit eu ligula consequat, dignissim elementum diam aliquet. Maecenas ipsum tellus, condimentum sit amet ultrices et, semper id nibh. Fusce varius porttitor dui vitae fermentum. Nam placerat, nunc eget tempus egestas, enim nisl scelerisque erat, in venenatis leo leo a leo. Nam quis nibh vitae turpis efficitur lobortis id a metus.

In aliquet dui sapien, ut semper elit sodales sed. Proin quis libero luctus libero scelerisque ornare eget id mi. Cras accumsan arcu ut ante pharetra fringilla. Sed feugiat placerat dolor, et feugiat lorem iaculis id. Cras interdum est nec elit molestie, sed aliquam quam hendrerit. Proin sit amet pellentesque enim. Nam bibendum, mauris vel eleifend ultricies, orci sapien hendrerit sem, a tristique lectus nulla a ligula.
"#,
    );

    let cid2= db.add_card(
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
    );

    let cid3=db.add_card("👻 oijwqwu qwdiowhq  i hio h qiowhqwheqw👻👻 wwq qiuwhdidwh👻👻👻❤️\n\nauhui ❤️awudhia\n🧑‍🌾❤️👨‍🦰jfpkw huiw wjwioj ijf weoijwioejfiowejfiowjfiowej\n\nthis\tis\ta\tparagraph\twith\ttabs\n\n```rust\nfn main() {\n\tprintln!(\"Hello, world!\");\n}\n```");

    let tid1 = db.add_tag("lorem").unwrap();
    let tid2 = db.add_tag("rust").unwrap();
    let tid3 = db.add_tag("image").unwrap();
    let tid4 = db.add_tag("abc").unwrap();
    let tid5 = db.add_tag("tagwithalongnamethatyoucantreadnoob").unwrap();
    let tid6 = db.add_tag("yoyo").unwrap();
    let tid7 = db.add_tag("zup").unwrap();
    let tid8 = db.add_tag("test").unwrap();
    let tid9 = db.add_tag("cba").unwrap();
    let tid10 = db
        .add_tag("anothertagwithalongnamethatyoucantread")
        .unwrap();

    db.add_tag_for_card(cid1, tid1);
    db.add_tag_for_card(cid2, tid2);
    db.add_tag_for_card(cid2, tid3);
}
