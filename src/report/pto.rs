use std::collections::BTreeMap;
use std::fmt::{self, Display};

use time::{Date, Duration};

use crate::entry::Entry;
use crate::iter::each_week;
use crate::timesource::TimeSource;

pub fn prepare<TS: TimeSource>(entries: Vec<Entry>, full_week: i64, ts: &TS) -> Report {
    let mut weeks = BTreeMap::new();
    for (week_start, entries) in each_week(entries, ts) {
        let start = week_start.midnight().assume_offset(ts.local_offset());
        let stop = start + Duration::week();
        let minutes = entries.iter().map(|e| e.minutes_between(start, stop)).sum();
        weeks.insert(week_start, minutes);
    }
    Report { weeks, full_week }
}

pub struct Report {
    weeks: BTreeMap<Date, i64>,
    full_week: i64,
}

impl Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut years: BTreeMap<i32, i64> = BTreeMap::new();
        for (week_start, minutes) in &self.weeks {
            let pto = 0.max(self.full_week - minutes);
            writeln!(f, "{} work={:4} pto={:4}", week_start, minutes, pto)?;
            let year_total = years.entry(week_start.year()).or_default();
            *year_total += pto;
        }
        if !years.is_empty() {
            writeln!(f)?;
            for (year, minutes) in years {
                writeln!(
                    f,
                    "{} total_pto={:5} days={:3}",
                    year,
                    minutes,
                    minutes / 60 / 8
                )?;
            }
        }
        Ok(())
    }
}
