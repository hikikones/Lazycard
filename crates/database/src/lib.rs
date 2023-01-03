use std::path::Path;

use rusqlite::{Connection, Params, Row};

mod database;

pub use database::*;

type SqliteId = i64;
type SqliteResult<T> = Result<T, rusqlite::Error>;

pub struct Sqlite(Connection);

impl Sqlite {
    fn open(path: impl AsRef<Path>) -> SqliteResult<Self> {
        let connection = Connection::open(path)?;
        Ok(Self(connection))
    }

    fn version(&self) -> SqliteId {
        self.fetch_one("SELECT user_version FROM pragma_user_version", [])
    }

    fn set_version(&self, version: SqliteId) {
        self.execute_one(&format!("PRAGMA user_version = {version}"), []);
    }

    pub fn execute_one(&self, sql: &str, params: impl Params) -> usize {
        self.0.execute(sql, params).unwrap()
    }

    pub fn execute_all(&self, sql: &str) {
        self.0.execute_batch(sql).unwrap();
    }

    pub fn fetch_one<T>(&self, sql: &str, params: impl Params) -> T
    where
        T: FromRow,
    {
        self.0
            .query_row(sql, params, |row| Ok(T::from_row(row)))
            .unwrap()
    }

    pub fn fetch_all<T>(&self, sql: &str, params: impl Params) -> Vec<T>
    where
        T: FromRow,
    {
        let mut statement = self.0.prepare(sql).unwrap();
        let rows = statement
            .query_map(params, |row| Ok(T::from_row(row)))
            .unwrap();

        let mut items = Vec::new();
        for row in rows {
            items.push(row.unwrap());
        }
        items
    }

    pub fn last_insert_rowid(&self) -> SqliteId {
        self.0.last_insert_rowid()
    }
}

pub trait FromRow: Sized {
    fn from_row(row: &Row) -> Self;
}

impl FromRow for SqliteId {
    fn from_row(row: &Row) -> Self {
        row.get(0).unwrap()
    }
}
