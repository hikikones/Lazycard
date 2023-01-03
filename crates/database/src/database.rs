use std::ops::Deref;

use chrono::NaiveDate;

use crate::*;

pub struct Database(Sqlite);

impl Database {
    pub fn open(path: impl AsRef<Path>) -> SqliteResult<Self> {
        let sqlite = Sqlite::open(path)?;

        const CURRENT_VERSION: SqliteId = 1;

        match sqlite.version() {
            0 => {
                sqlite.execute_all(include_str!("schema.sql"));
                sqlite.set_version(CURRENT_VERSION);
            }
            CURRENT_VERSION => {}
            _ => todo!(),
        }

        Ok(Self(sqlite))
    }
}

impl Deref for Database {
    type Target = Sqlite;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Card {
    pub id: SqliteId,
    pub content: String,
    pub review: CardReview,
}

pub struct CardReview {
    pub due_date: NaiveDate,
    pub due_days: usize,
    pub recall_attempts: usize,
    pub successful_recalls: usize,
}

pub struct Tag {
    pub id: SqliteId,
    pub name: String,
}

impl FromRow for Card {
    fn from_row(row: &Row) -> Self {
        Self {
            id: row.get(0).unwrap(),
            content: row.get(1).unwrap(),
            review: CardReview {
                due_date: row.get(2).unwrap(),
                due_days: row.get(3).unwrap(),
                recall_attempts: row.get(4).unwrap(),
                successful_recalls: row.get(5).unwrap(),
            },
        }
    }
}

impl FromRow for Tag {
    fn from_row(row: &Row) -> Self {
        Self {
            id: row.get(0).unwrap(),
            name: row.get(1).unwrap(),
        }
    }
}
