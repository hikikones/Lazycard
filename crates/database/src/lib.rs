use std::path::Path;

use rusqlite::{Connection, Params, Row};

mod data;

pub use data::*;

type Id = i64;
type ModifiedRows = usize;
type DbResult<T> = Result<T, rusqlite::Error>;

pub struct Database(Connection);

impl Database {
    pub fn open(path: impl AsRef<Path>) -> DbResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self(conn);

        const CURRENT_VERSION: Id = 1;

        match db.version() {
            0 => {
                db.execute_all(include_str!("schema.sql")).unwrap();
                db.set_version(CURRENT_VERSION);
            }
            CURRENT_VERSION => {}
            _ => todo!(),
        }

        Ok(db)
    }

    pub fn version(&self) -> Id {
        self.query_single("SELECT user_version FROM pragma_user_version", [])
            .unwrap()
    }

    pub fn set_version(&self, version: Id) {
        self.execute_single(&format!("PRAGMA user_version = {version}"), [])
            .unwrap();
    }

    pub fn execute_single(&self, sql: &str, params: impl Params) -> DbResult<ModifiedRows> {
        self.0.execute(sql, params)
    }

    pub fn execute_all(&self, sql: &str) -> DbResult<()> {
        self.0.execute_batch(sql)
    }

    pub fn query_single<T>(&self, sql: &str, params: impl Params) -> DbResult<T>
    where
        T: FromRow,
    {
        self.0.query_row(sql, params, |row| T::from_row(row))
    }

    pub fn query_all<T>(&self, sql: &str, params: impl Params) -> DbResult<Vec<T>>
    where
        T: FromRow,
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
