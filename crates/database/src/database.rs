use std::{num::ParseIntError, ops::Deref, path::Path, str::FromStr};

use chrono::NaiveDateTime;
use rusqlite::{
    types::{FromSql, ToSqlOutput, Value, ValueRef},
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
    pub due_days: usize,
    pub card_id: SqliteId,
}

pub struct Tag {
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
