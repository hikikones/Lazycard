use std::{marker::PhantomData, path::Path};

use rusqlite::{Connection, Params, Row};

pub type SqliteId = i64;
pub type SqliteResult<T> = Result<T, rusqlite::Error>;

pub struct Sqlite(Connection);

pub trait FromRow {
    fn from_row(row: &Row) -> Self;
}

impl Sqlite {
    pub(crate) fn open(path: impl AsRef<Path>) -> SqliteResult<Self> {
        let connection = Connection::open(path)?;
        Ok(Self(connection))
    }

    pub(crate) fn version(&self) -> SqliteId {
        self.fetch("SELECT user_version FROM pragma_user_version")
            .single()
    }

    pub(crate) fn set_version(&self, version: SqliteId) {
        self.execute(&format!("PRAGMA user_version = {version}"))
            .single();
    }

    pub fn execute<'a>(&'a self, sql: &'a str) -> Execute {
        Execute {
            sql,
            connection: &self.0,
        }
    }

    pub fn fetch<'a, T>(&'a self, sql: &'a str) -> Fetch<T>
    where
        T: FromRow,
    {
        Fetch {
            sql,
            connection: &self.0,
            data: PhantomData,
        }
    }

    pub fn last_insert_rowid(&self) -> SqliteId {
        self.0.last_insert_rowid()
    }
}

pub struct Execute<'a> {
    sql: &'a str,
    connection: &'a Connection,
}

impl<'a> Execute<'a> {
    pub fn single(self) -> usize {
        self.connection.execute(self.sql, []).unwrap()
    }

    pub fn single_with_params(self, params: impl Params) -> usize {
        self.connection.execute(self.sql, params).unwrap()
    }

    pub fn all(self) {
        self.connection.execute_batch(self.sql).unwrap();
    }
}

pub struct Fetch<'a, T>
where
    T: FromRow,
{
    sql: &'a str,
    connection: &'a Connection,
    data: PhantomData<T>,
}

impl<'a, T> Fetch<'a, T>
where
    T: FromRow,
{
    pub fn single(self) -> T {
        self._single([]).unwrap()
    }

    pub fn single_with_params(self, params: impl Params) -> T {
        self._single(params).unwrap()
    }

    pub fn get_single(self) -> Option<T> {
        self._single([]).ok()
    }

    pub fn get_single_with_params(self, params: impl Params) -> Option<T> {
        self._single(params).ok()
    }

    pub fn all(self) -> Vec<T> {
        self._all([]).unwrap()
    }

    pub fn all_with_params(self, params: impl Params) -> Vec<T> {
        self._all(params).unwrap()
    }

    fn _single(self, params: impl Params) -> SqliteResult<T> {
        self.connection
            .prepare(self.sql)?
            .query_row(params, |row| Ok(T::from_row(row)))
    }

    fn _all(self, params: impl Params) -> SqliteResult<Vec<T>> {
        let mut statement = self.connection.prepare(self.sql)?;
        let rows = statement.query_map(params, |row| Ok(T::from_row(row)))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.unwrap());
        }

        Ok(items)
    }
}

impl FromRow for SqliteId {
    fn from_row(row: &Row) -> Self {
        row.get(0).unwrap()
    }
}

impl FromRow for (SqliteId, String) {
    fn from_row(row: &Row) -> Self {
        (row.get(0).unwrap(), row.get(1).unwrap())
    }
}
