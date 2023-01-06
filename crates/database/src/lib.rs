use std::{marker::PhantomData, path::Path};

use rusqlite::{Connection, Params, Row, Statement};

pub use rusqlite::params;

mod database;

pub use database::*;

pub type SqliteId = i64;
pub type SqliteResult<T> = Result<T, rusqlite::Error>;

pub struct Sqlite(Connection);

impl Sqlite {
    fn open(path: impl AsRef<Path>) -> SqliteResult<Self> {
        let connection = Connection::open(path)?;
        Ok(Self(connection))
    }

    fn version(&self) -> SqliteId {
        self.fetch("SELECT user_version FROM pragma_user_version")
            .single()
    }

    fn set_version(&self, version: SqliteId) {
        self.execute(&format!("PRAGMA user_version = {version}"))
            .single();
    }

    pub fn execute<'a>(&'a self, sql: &'a str) -> Execute {
        Execute {
            sql,
            executer: Executer {
                connection: &self.0,
            },
        }
    }

    pub fn fetch<T>(&self, sql: &str) -> Fetch<T>
    where
        T: FromRow,
    {
        Fetch {
            fetcher: Fetcher {
                statement: self.0.prepare(sql).unwrap(),
                data: PhantomData,
            },
        }
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

impl FromRow for (SqliteId, String) {
    fn from_row(row: &Row) -> Self {
        (row.get(0).unwrap(), row.get(1).unwrap())
    }
}

pub struct Execute<'a> {
    sql: &'a str,
    executer: Executer<'a>,
}

impl<'a> Execute<'a> {
    pub fn single(self) -> usize {
        self.executer.single(self.sql, [])
    }

    pub fn all(self) {
        self.executer.all(self.sql)
    }

    pub fn args<P>(self, args: P) -> ExecuteArgs<'a, P>
    where
        P: Params,
    {
        ExecuteArgs {
            sql: self.sql,
            executer: self.executer,
            args,
        }
    }
}

pub struct ExecuteArgs<'a, P>
where
    P: Params,
{
    sql: &'a str,
    executer: Executer<'a>,
    args: P,
}

impl<'a, P> ExecuteArgs<'a, P>
where
    P: Params,
{
    pub fn single(self) -> usize {
        self.executer.single(self.sql, self.args)
    }
}

struct Executer<'a> {
    connection: &'a Connection,
}

impl<'a> Executer<'a> {
    fn single(self, sql: &str, params: impl Params) -> usize {
        self.connection.execute(sql, params).unwrap()
    }

    fn all(self, sql: &str) {
        self.connection.execute_batch(sql).unwrap();
    }
}

pub struct Fetch<'a, T>
where
    T: FromRow,
{
    fetcher: Fetcher<'a, T>,
}

impl<'a, T> Fetch<'a, T>
where
    T: FromRow,
{
    pub fn single(self) -> T {
        self.fetcher.single([])
    }

    pub fn all(self) -> Vec<T> {
        self.fetcher.all([])
    }

    pub fn args<P>(self, args: P) -> FetchArgs<'a, T, P>
    where
        P: Params,
    {
        FetchArgs {
            fetcher: self.fetcher,
            args,
        }
    }
}

pub struct FetchArgs<'a, T, P>
where
    T: FromRow,
    P: Params,
{
    fetcher: Fetcher<'a, T>,
    args: P,
}

impl<'a, T, P> FetchArgs<'a, T, P>
where
    T: FromRow,
    P: Params,
{
    pub fn single(self) -> T {
        self.fetcher.single(self.args)
    }

    pub fn all(self) -> Vec<T> {
        self.fetcher.all(self.args)
    }
}

struct Fetcher<'a, T>
where
    T: FromRow,
{
    statement: Statement<'a>,
    data: PhantomData<T>,
}

impl<'a, T> Fetcher<'a, T>
where
    T: FromRow,
{
    pub fn single(mut self, params: impl Params) -> T {
        self.statement
            .query_row(params, |row| Ok(T::from_row(row)))
            .unwrap()
    }

    pub fn all(mut self, params: impl Params) -> Vec<T> {
        let rows = self
            .statement
            .query_map(params, |row| Ok(T::from_row(row)))
            .unwrap();

        let mut items = Vec::new();
        for row in rows {
            items.push(row.unwrap());
        }
        items
    }
}
