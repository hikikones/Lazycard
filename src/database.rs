use std::{marker::PhantomData, path::Path};

use rusqlite::{Connection, Params, Row, Statement};

pub struct Database(Connection);

type Id = i64;

pub trait FromRow: Sized {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error>;
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;

        Ok(Self(conn))
    }

    pub fn version(&self) -> Id {
        self.fetch("SELECT user_version FROM pragma_user_version")
            .unwrap()
            .single()
            .unwrap()
    }

    pub fn set_version(&self, version: Id) {
        todo!()
    }

    pub fn execute(&self, sql: &str, args: impl Params) -> Result<usize, rusqlite::Error> {
        self.0.execute(sql, args)
    }

    pub fn fetch<T>(&self, sql: &str) -> Result<Fetch<'_, T>, rusqlite::Error>
    where
        T: FromRow,
    {
        let statement = self.0.prepare(sql)?;
        Ok(Fetch {
            fetcher: Fetcher {
                statement,
                data: PhantomData,
            },
        })
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
    pub fn single(self) -> Result<T, rusqlite::Error> {
        self.fetcher.single([])
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
    pub fn single(self) -> Result<T, rusqlite::Error> {
        self.fetcher.single(self.args)
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
    pub fn single(mut self, params: impl Params) -> Result<T, rusqlite::Error> {
        self.statement.query_row(params, |row| T::from_row(row))
    }
}

impl FromRow for i64 {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        row.get(0)
    }
}
