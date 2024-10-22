use std::error::Error;
use std::path::PathBuf;

use time::{Duration, OffsetDateTime};

use crate::entry::{into_time_entries, Entry, TimeEntry};
use crate::extents;
use crate::file::{read_entries, read_last_entries, t_open};
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
    pub fn tail(&self) -> Result<EntriesResult, Box<dyn Error>> {
        let entries = self.read_last_entries(100)?;
        let entries = into_time_entries(entries);
        Ok(EntriesResult { entries })
    }

    pub fn all(&self) -> Result<EntriesResult, Box<dyn Error>> {
        let entries = self.read_entries()?;
        let entries = into_time_entries(entries);
        Ok(EntriesResult { entries })
    }

    fn read_last_entries(&self, n: u64) -> Result<Vec<Entry>, Box<dyn std::error::Error>> {
        match &self.tf {
            None => read_last_entries(n, &self.ts),
            Some(tf) => t_open(tf)?.read_last_entries(n, &self.ts),
        }
    }

    fn read_entries(&self) -> Result<Vec<Entry>, Box<dyn std::error::Error>> {
        match &self.tf {
            None => read_entries(&self.ts),
            Some(tf) => t_open(tf)?.read_entries(&self.ts),
        }
    }
}

pub struct EntriesResult {
    entries: Vec<TimeEntry>,
}

impl EntriesResult {
    pub fn is_working(&self) -> bool {
        match self.entries.last() {
            None => false,
            Some(e) => e.stop.is_none(),
        }
    }

    pub fn last_update(&self) -> Option<String> {
        match self.entries.last() {
            None => None,
            Some(e) => match &e.stop {
                None => Some(format!("{}", e.start)),
                Some(t) => Some(format!("{}", t)),
            },
        }
    }

    pub fn minutes_between(&self, range: (time::OffsetDateTime, time::OffsetDateTime)) -> i64 {
        minutes_between(&self.entries, range.0, range.1)
    }

    pub fn between(&self, range: (time::OffsetDateTime, time::OffsetDateTime)) -> Vec<TimeEntry> {
        entries_between(&self.entries, range.0, range.1)
    }

    pub fn recent_weeks(&self, previous_weeks: i16) -> Vec<PreviousWeek<'_>> {
        let (start_week, now) = extents::this_week();
        (0..previous_weeks)
            .rev()
            .map(|off| {
                let off = Duration::weeks((1 + off) as i64);
                PreviousWeek {
                    start: start_week - off,
                    todayish: now - off,
                    entries: &self.entries,
                }
            })
            .collect()
    }
}

pub struct PreviousWeek<'a> {
    pub start: time::OffsetDateTime,
    todayish: time::OffsetDateTime,
    entries: &'a [TimeEntry],
}

impl<'a> PreviousWeek<'a> {
    pub fn minutes_to_date(&self) -> i64 {
        minutes_between(self.entries, self.start, self.todayish)
    }

    pub fn total_minutes(&self) -> i64 {
        minutes_between(self.entries, self.start, self.start + Duration::days(7))
    }
}

fn minutes_between(entries: &[TimeEntry], start: OffsetDateTime, stop: OffsetDateTime) -> i64 {
    entries
        .iter()
        .fold(0, |sum, entry| sum + entry.minutes_between(start, stop))
}

pub fn entries_between(
    entries: &[TimeEntry],
    start: OffsetDateTime,
    stop: OffsetDateTime,
) -> Vec<TimeEntry> {
    entries
        .iter()
        .filter(|e| e.overlaps(start, stop))
        .cloned()
        .collect()
}
