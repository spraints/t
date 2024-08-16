use time::OffsetDateTime;

use crate::entry::{into_time_entries, TimeEntry};
use crate::extents;
use crate::file::read_last_entries;
use crate::timesource::TimeSource;

pub fn for_cli<TS>(ts: TS) -> Context<TS> {
    Context { ts }
}

pub struct Context<TS> {
    ts: TS,
}

impl<TS: TimeSource> Context<TS> {
    pub fn status(&self) -> Result<Status, Box<dyn std::error::Error>> {
        let entries = read_last_entries(100, &self.ts)?;
        let entries = into_time_entries(entries);
        Ok(Status { entries })
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
