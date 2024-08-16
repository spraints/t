use std::path::PathBuf;

use time::OffsetDateTime;

use crate::entry::{into_time_entries, Entry, TimeEntry};
use crate::extents;
use crate::file::{read_last_entries, t_open};
use crate::timesource::TimeSource;

pub fn for_cli<TS>(ts: TS) -> Context<TS> {
    Context { ts, tf: None }
}

pub fn for_web<TS>(tf: PathBuf, ts: TS) -> Context<TS> {
    Context { ts, tf: Some(tf) }
}

pub struct Context<TS> {
    ts: TS,
    tf: Option<PathBuf>,
}

impl<TS: TimeSource> Context<TS> {
    pub fn status(&self) -> Result<Status, Box<dyn std::error::Error>> {
        let entries = self.read_last_entries(100)?;
        let entries = into_time_entries(entries);
        Ok(Status { entries })
    }

    fn read_last_entries(&self, n: u64) -> Result<Vec<Entry>, Box<dyn std::error::Error>> {
        match &self.tf {
            None => read_last_entries(n, &self.ts),
            Some(tf) => t_open(&tf)?.read_last_entries(n, &self.ts),
        }
    }
}

pub struct Status {
    entries: Vec<TimeEntry>,
}

impl Status {
    pub fn is_working(&self) -> bool {
        match self.entries.last() {
            None => false,
            Some(e) => e.stop.is_none(),
        }
    }

    pub fn minutes_this_week(&self) -> i64 {
        let (start_week, now) = extents::this_week();
        minutes_between(&self.entries, start_week, now)
    }
}

fn minutes_between(entries: &[TimeEntry], start: OffsetDateTime, stop: OffsetDateTime) -> i64 {
    entries
        .iter()
        .fold(0, |sum, entry| sum + entry.minutes_between(start, stop))
}
