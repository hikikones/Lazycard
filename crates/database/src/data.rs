use chrono::NaiveDateTime;

use crate::*;

pub struct Card {
    pub id: Id,
    pub content: String,
    pub review: CardReview,
}

pub struct CardReview {
    pub due_date: NaiveDateTime,
    pub due_days: usize,
    pub recall_attempts: usize,
    pub successful_recalls: usize,
}

pub struct Tag {
    pub id: Id,
    pub name: String,
}

impl FromRow for Card {
    fn from_row(row: &Row) -> DbResult<Self> {
        let id = row.get(0)?;
        let content = row.get(1)?;
        let due_date = row.get(2)?;
        let due_days = row.get(3)?;
        let recall_attempts = row.get(4)?;
        let successful_recalls = row.get(5)?;

        Ok(Self {
            id,
            content,
            review: CardReview {
                due_date,
                due_days,
                recall_attempts,
                successful_recalls,
            },
        })
    }
}

impl FromRow for Tag {
    fn from_row(row: &Row) -> DbResult<Self> {
        let id = row.get(0)?;
        let name = row.get(1)?;

        Ok(Self { id, name })
    }
}
