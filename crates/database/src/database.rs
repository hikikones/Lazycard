use std::{num::ParseIntError, ops::Deref, path::Path, str::FromStr};

use chrono::NaiveDateTime;
use rusqlite::{
    types::{FromSql, ToSqlOutput, Value, ValueRef},
    Connection, Row, ToSql,
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
            sqlite.execute(include_str!("schema.sql")).all();
            sqlite.set_version(CURRENT_VERSION);
        }
        _ => todo!(),
    }
}

pub struct Card {
    pub id: SqliteId,
    pub content: String,
    pub review: CardReview,
}

pub struct CardReview {
    pub due_date: NaiveDateTime,
    pub due_days: usize,
    pub recall_attempts: usize,
    pub successful_recalls: usize,
}

pub struct Tag {
    pub name: String,
}

pub struct Media {
    pub seahash: Seahash,
    pub bytes: Vec<u8>,
    pub file_ext: String,
}

impl FromRow for Card {
    fn from_row(row: &Row) -> Self {
        Self {
            id: row.get(0).unwrap(),
            content: row.get(1).unwrap(),
            review: CardReview {
                due_date: row.get(2).unwrap(),
                due_days: row.get(3).unwrap(),
                recall_attempts: row.get(4).unwrap(),
                successful_recalls: row.get(5).unwrap(),
            },
        }
    }
}

impl FromRow for Tag {
    fn from_row(row: &Row) -> Self {
        Self {
            name: row.get(0).unwrap(),
        }
    }
}

impl FromRow for Media {
    fn from_row(row: &Row) -> Self {
        Self {
            seahash: row.get(0).unwrap(),
            bytes: row.get(1).unwrap(),
            file_ext: row.get(2).unwrap(),
        }
    }
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

impl ToSql for Seahash {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Blob(
            self.0.to_be_bytes().to_vec(),
        )))
    }
}

impl FromSql for Seahash {
    fn column_result(value: ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let bytes: [u8; 8] = value.as_blob()?.try_into().unwrap();
        Ok(Seahash(u64::from_be_bytes(bytes)))
    }
}

impl FromRow for Seahash {
    fn from_row(row: &Row) -> Self {
        row.get(0).unwrap()
    }
}

impl FromRow for (Seahash, String) {
    fn from_row(row: &Row) -> Self {
        (row.get(0).unwrap(), row.get(1).unwrap())
    }
}

impl FromRow for (Seahash, Vec<u8>) {
    fn from_row(row: &Row) -> Self {
        (row.get(0).unwrap(), row.get(1).unwrap())
    }
}