use rusqlite::{Connection, Params, Row};

pub type SqliteId = i64;
pub type SqliteResult<T> = Result<T, rusqlite::Error>;

pub struct Sqlite(pub(crate) Connection);

impl Sqlite {
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

    pub fn execute_one(&self, sql: &str, params: impl Params) -> SqliteResult<usize> {
        self.0.execute(sql, params)
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

    pub fn fetch_iter<T>(
        &self,
        sql: &str,
        args: impl Params,
        f: impl Fn(&Row),
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
