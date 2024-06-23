use std::path::Path;

use rusqlite::Connection;

pub use rusqlite::{params, params_from_iter, types::*, OpenFlags, Params, Row};

pub type SqliteId = i64;
pub type SqliteResult<T> = Result<T, rusqlite::Error>;

pub struct Sqlite(Connection);

impl Sqlite {
    pub fn open(path: impl AsRef<Path>) -> SqliteResult<Self> {
        let connection = Connection::open(path)?;
        Ok(Self(connection))
    }

    pub fn open_with_flags(path: impl AsRef<Path>, flags: OpenFlags) -> SqliteResult<Self> {
        let connection = Connection::open_with_flags(path, flags)?;
        Ok(Self(connection))
    }

    pub fn path(&self) -> Option<&str> {
        self.0.path()
    }

    pub fn version(&self) -> SqliteId {
        self.0
            .query_row("SELECT user_version FROM pragma_user_version", [], |row| {
                row.get(0)
            })
            .unwrap()
    }

    pub fn set_version(&self, version: SqliteId) {
        self.0
            .execute(&format!("PRAGMA user_version = {version}"), [])
            .unwrap();
    }

    pub fn execute_one(&self, sql: &str, args: impl Params) -> SqliteResult<usize> {
        self.0.execute(sql, args)
    }

    pub fn execute_all(&self, sql: &str) -> SqliteResult<()> {
        self.0.execute_batch(sql)
    }

    pub fn fetch_one<T>(
        &self,
        sql: &str,
        args: impl Params,
        f: impl FnOnce(&Row) -> SqliteResult<T>,
    ) -> SqliteResult<T> {
        self.0.query_row(sql, args, f)
    }

    pub fn fetch_one_maybe<T>(
        &self,
        sql: &str,
        args: impl Params,
        f: impl FnOnce(&Row) -> SqliteResult<T>,
    ) -> SqliteResult<Option<T>> {
        let mut stmt = self.0.prepare(sql)?;
        let mut rows = stmt.query(args)?;

        if let Ok(Some(row)) = rows.next() {
            return Ok(Some(f(row)?));
        }

        Ok(None)
    }

    pub fn fetch_all<T>(
        &self,
        sql: &str,
        args: impl Params,
        mut f: impl FnMut(&Row) -> SqliteResult<T>,
    ) -> SqliteResult<Vec<T>> {
        let mut statement = self.0.prepare(sql)?;
        let rows = statement.query_map(args, |row| f(row))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }

        Ok(items)
    }

    pub fn fetch_with(
        &self,
        sql: &str,
        args: impl Params,
        mut f: impl FnMut(&Row),
    ) -> SqliteResult<()> {
        let mut statement = self.0.prepare(sql)?;
        let mut rows = statement.query(args)?;

        while let Ok(Some(row)) = rows.next() {
            f(row);
        }

        Ok(())
    }

    pub fn last_insert_rowid(&self) -> SqliteId {
        self.0.last_insert_rowid()
    }
}
