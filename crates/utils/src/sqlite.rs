use std::path::Path;

use rusqlite::{Connection, OptionalExtension};

pub use rusqlite::{Params, Row, types::*};

pub type SqliteId = i64;
pub type SqliteError = rusqlite::Error;
pub type SqliteResult<T> = Result<T, SqliteError>;

pub struct Sqlite(Connection);

impl Sqlite {
    pub fn open(path: impl AsRef<Path>) -> SqliteResult<Self> {
        Ok(Self(Connection::open(path)?))
    }

    pub fn open_in_memory() -> SqliteResult<Self> {
        Ok(Self(Connection::open_in_memory()?))
    }

    pub fn path(&self) -> Option<&str> {
        self.0.path()
    }

    pub fn version(&self) -> SqliteId {
        self.0
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap()
    }

    pub fn set_version(&self, version: SqliteId) {
        self.0.pragma_update(None, "user_version", version).unwrap();
    }

    pub fn execute(&self, sql: &str, args: impl Params) -> SqliteResult<usize> {
        self.0.execute(sql, args)
    }

    pub fn execute_batch(&self, sql: &str) -> SqliteResult<()> {
        self.0.execute_batch(sql)
    }

    pub fn query<T>(
        &self,
        sql: &str,
        args: impl Params,
        mut f: impl FnMut(&Row) -> SqliteResult<T>,
    ) -> SqliteResult<()> {
        let mut statement = self.0.prepare(sql)?;
        let mut rows = statement.query(args)?;

        while let Some(row) = rows.next()? {
            f(row)?;
        }

        Ok(())
    }

    pub fn query_first<T>(
        &self,
        sql: &str,
        args: impl Params,
        f: impl FnOnce(&Row) -> SqliteResult<T>,
    ) -> SqliteResult<Option<T>> {
        self.0.query_row(sql, args, f).optional()
    }

    pub fn query_single<T>(
        &self,
        sql: &str,
        args: impl Params,
        f: impl FnOnce(&Row) -> SqliteResult<T>,
    ) -> SqliteResult<T> {
        self.0.query_one(sql, args, f)
    }

    pub fn last_insert_rowid(&self) -> SqliteId {
        self.0.last_insert_rowid()
    }
}
