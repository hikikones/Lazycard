use std::{num::ParseIntError, ops::Deref, path::Path, str::FromStr};

use chrono::NaiveDateTime;
use rusqlite::{
    params_from_iter,
    types::{FromSql, FromSqlResult, ToSqlOutput, Value, ValueRef},
    Connection, ToSql,
};

use crate::sqlite::*;

pub struct Database(Option<Sqlite>);

impl Database {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn open(&mut self, path: impl AsRef<Path>) -> SqliteResult<()> {
        let connection = Connection::open(path)?;
        let sqlite = Sqlite(connection);

        migrate(&sqlite);

        self.0 = Some(sqlite);

        Ok(())
    }

    pub fn get_cards(&self) -> Vec<Card> {
        self.fetch_all("SELECT * FROM cards", [], |row| {
            Ok(Card {
                id: row.get(0)?,
                content: row.get(1)?,
            })
        })
        .unwrap()
    }

    pub fn get_tags(&self) -> Vec<Tag> {
        self.fetch_all("SELECT * FROM tags", [], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .unwrap()
    }

    pub fn get_cards_with_tags(&self, tags: impl AsRef<[SqliteId]>) -> Vec<Card> {
        let tags = tags.as_ref();

        if tags.is_empty() {
            return self.get_cards();
        }

        let sql = format!(
            "
            SELECT * FROM cards c \
            JOIN card_tag ct ON ct.card_id = c.id WHERE tag_id IN ({}) \
            GROUP BY ct.card_id HAVING Count(*) = {}
            ",
            tags.iter().map(|_| "?").collect::<Box<[_]>>().join(","),
            tags.len()
        );

        self.fetch_all(&sql, params_from_iter(tags.iter()), |row| {
            Ok(Card {
                id: row.get(0)?,
                content: row.get(1)?,
            })
        })
        .unwrap()
    }

    pub fn get_cards_without_tags(&self) -> Vec<Card> {
        self.fetch_all(
            "
            SELECT * FROM cards c WHERE NOT EXISTS ( \
                SELECT ct.card_id FROM card_tag ct \
                WHERE c.id = ct.card_id \
            )",
            [],
            |row| {
                Ok(Card {
                    id: row.get(0)?,
                    content: row.get(1)?,
                })
            },
        )
        .unwrap()
    }
}

impl Deref for Database {
    type Target = Sqlite;

    fn deref(&self) -> &Self::Target {
        &self.0.as_ref().unwrap()
    }
}

fn migrate(sqlite: &Sqlite) {
    const CURRENT_VERSION: SqliteId = 1;

    match sqlite.version() {
        CURRENT_VERSION => {}
        0 => {
            sqlite.execute_all(include_str!("schema.sql")).unwrap();
            sqlite.set_version(CURRENT_VERSION);
        }
        _ => todo!(),
    }
}

pub struct Card {
    pub id: SqliteId,
    pub content: String,
}

pub struct Schedule {
    pub id: SqliteId,
    pub due_date: NaiveDateTime,
    pub due_days: u32,
    pub card_id: SqliteId,
}

pub struct Review {
    pub id: SqliteId,
    pub date: NaiveDateTime,
    pub success: bool,
    pub card_id: SqliteId,
}

pub struct Tag {
    pub id: SqliteId,
    pub name: String,
}

pub struct Asset {
    pub seahash: Seahash,
    pub bytes: Vec<u8>,
    pub extension: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Seahash(u64);

impl Seahash {
    pub fn from_raw(hash: u64) -> Self {
        Self(hash)
    }

    pub fn raw(self) -> u64 {
        self.0
    }
}

impl From<&[u8]> for Seahash {
    fn from(buf: &[u8]) -> Self {
        Self(seahash::hash(buf))
    }
}

impl FromStr for Seahash {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hash = s.parse::<u64>()?;
        Ok(Self(hash))
    }
}

impl FromSql for Seahash {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let bytes: [u8; 8] = value.as_blob()?.try_into().unwrap();
        Ok(Seahash(u64::from_be_bytes(bytes)))
    }
}

impl ToSql for Seahash {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Blob(
            self.0.to_be_bytes().to_vec(),
        )))
    }
}
