use std::path::Path;

use rusqlite::{Connection, Params, Row};

type Id = i64;
type ModifiedRows = usize;
type DbResult<T> = Result<T, rusqlite::Error>;

pub struct Database(Connection);

impl Database {
    pub fn open(path: impl AsRef<Path>) -> DbResult<Self> {
        let conn = Connection::open(path)?;

        Ok(Self(conn))
    }

    pub fn version(&self) -> Id {
        self.query_single("SELECT user_version FROM pragma_user_version", [])
            .unwrap()
    }

    pub fn set_version(&self, version: Id) {
        self.execute_single("PRAGMA user_version = ?", [version])
            .unwrap();
    }

    pub fn execute_single<P>(&self, sql: &str, params: P) -> DbResult<ModifiedRows>
    where
        P: Params,
    {
        self.0.execute(sql, params)
    }

    pub fn execute_all<P>(&self, sql: &str) -> DbResult<()>
    where
        P: Params,
    {
        self.0.execute_batch(sql)
    }

    pub fn query_single<T, P>(&self, sql: &str, params: P) -> DbResult<T>
    where
        T: FromRow,
        P: Params,
    {
        self.0.query_row(sql, params, |row| T::from_row(row))
    }

    pub fn query_all<T, P>(&self, sql: &str, params: P) -> DbResult<Vec<T>>
    where
        T: FromRow,
        P: Params,
    {
        let mut statement = self.0.prepare(sql)?;
        let rows = statement.query_map(params, |row| T::from_row(row))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }

        Ok(items)
    }

    pub fn last_insert_rowid(&self) -> Id {
        self.0.last_insert_rowid()
    }
}

pub trait FromRow: Sized {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error>;
}

impl FromRow for Id {
    fn from_row(row: &Row) -> DbResult<Self> {
        row.get(0)
    }
}
