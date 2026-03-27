use serde::{Deserialize, Serialize};

use crate::UnixTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub content: String,
    pub creation_time: UnixTime,
    pub last_review_time: Option<UnixTime>,
    pub review_interval: f32,
    pub review_stability: f32,
    pub review_difficulty: f32,
    pub archived: bool,
}

impl Card {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            creation_time: UnixTime::now(),
            last_review_time: None,
            review_interval: 0.0,
            review_stability: 0.0,
            review_difficulty: 0.0,
            archived: false,
        }
    }

    pub fn is_due(&self, now: UnixTime) -> bool {
        let review_time = self.last_review_time.unwrap_or(self.creation_time);
        let due_time = review_time.add_days(self.review_interval);
        due_time <= now
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CardId(pub(crate) u64); // todo: seahash?
