use utils::sqlite::*;

pub(crate) struct Scheduler(fsrs::FSRS);

pub(crate) struct CurrentReviewState {
    pub(crate) stability: f32,
    pub(crate) difficulty: f32,
    pub(crate) last_review_time: UnixTime,
}

pub(crate) struct NextReviewState {
    pub(crate) stability: f32,
    pub(crate) difficulty: f32,
    pub(crate) due_time: UnixTime,
}

impl Scheduler {
    pub(crate) fn new() -> Self {
        Self(fsrs::FSRS::default())
    }

    pub(crate) fn schedule(
        &self,
        state: CurrentReviewState,
        success: bool,
        desired_retention: f32,
    ) -> NextReviewState {
        let memory_state = if state.stability == 0.0 || state.difficulty == 0.0 {
            None
        } else {
            Some(fsrs::MemoryState {
                stability: state.stability,
                difficulty: state.difficulty,
            })
        };
        let now = UnixTime::now();
        let days_since_last_review = state.last_review_time.days_since(now);
        let next_states = self
            .0
            .next_states(memory_state, desired_retention, days_since_last_review)
            .unwrap();
        let new_state = if success {
            next_states.good
        } else {
            next_states.again
        };

        NextReviewState {
            stability: new_state.memory.stability,
            difficulty: new_state.memory.difficulty,
            due_time: now.add_days(new_state.interval.round().max(1.0)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl ToSql for UnixTime {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Integer(self.0 as i64)))
    }
}

impl FromSql for UnixTime {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_i64().map(|n| Self(n as u64))
    }
}
