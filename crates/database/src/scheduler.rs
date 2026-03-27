use serde::{Deserialize, Serialize};

use crate::Card;

pub(crate) struct Scheduler {
    fsrs: fsrs::FSRS,
    desired_retention: f32,
}

impl Scheduler {
    pub(crate) fn new(desired_retention: f32) -> Self {
        Self {
            fsrs: fsrs::FSRS::new(Some(&fsrs::DEFAULT_PARAMETERS)).unwrap(),
            desired_retention,
        }
    }

    pub(crate) fn schedule(&mut self, card: &mut Card, success: bool) {
        let current_memory_state = if card.review_stability == 0.0 || card.review_difficulty == 0.0
        {
            None
        } else {
            Some(fsrs::MemoryState {
                stability: card.review_stability,
                difficulty: card.review_difficulty,
            })
        };
        let now = UnixTime::now();
        let days_since_last_review = if let Some(last_review_time) = card.last_review_time {
            last_review_time.days_since(now)
        } else {
            0
        };
        let next_states = self
            .fsrs
            .next_states(
                current_memory_state,
                self.desired_retention,
                days_since_last_review,
            )
            .unwrap();
        let state = if success {
            next_states.good
        } else {
            next_states.again
        };

        card.last_review_time = now.into();
        card.review_interval = state.interval;
        card.review_stability = state.memory.stability;
        card.review_difficulty = state.memory.difficulty;
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UnixTime(u64);

impl UnixTime {
    const SECONDS_PER_DAY: u64 = 86400;

    pub fn now() -> Self {
        let secs_since_unix_epoch = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self(secs_since_unix_epoch)
    }

    pub(crate) const fn days_since(self, other: Self) -> u32 {
        (self.0.abs_diff(other.0) / Self::SECONDS_PER_DAY) as u32
    }

    pub(crate) const fn add_days(self, days: f32) -> Self {
        let days_in_secs = days * Self::SECONDS_PER_DAY as f32;
        Self(self.0 + days_in_secs as u64)
    }
}
