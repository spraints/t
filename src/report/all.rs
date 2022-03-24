use crate::entry::Entry;
use crate::iter::{each_day_in_week, each_week};
use crate::timesource::TimeSource;
use std::fmt::Debug;
use time::{Date, Duration, OffsetDateTime};

#[derive(Debug, PartialEq)]
pub struct All<T: PartialEq> {
    pub start: Date,
    pub minutes: i64,
    pub segments: usize,
    pub analysis: Option<AllAnalysis<T>>,
}

#[derive(Debug, PartialEq)]
pub struct AllAnalysis<T: PartialEq> {
    pub min: i64,
    pub max: i64,
    pub mean: i64,
    pub stddev: i64,
    pub sparks: Vec<Vec<T>>,
}

pub fn calc<T: PartialEq + Copy, TS: TimeSource>(
    entries: Vec<Entry>,
    sparks: &[T],
    ts: &TS,
) -> Vec<All<T>> {
    each_week(entries, ts)
        .map(|(start, entries)| calc_all_week(start, entries, sparks, ts))
        .collect()
}

const ONE_DAY: Duration = Duration::days(1);
const SUNDAY_TO_SATURDAY: Duration = Duration::days(6);

fn calc_all_week<T: PartialEq + Copy, TS: TimeSource>(
    start: Date,
    entries: Vec<Entry>,
    sparks: &[T],
    ts: &TS,
) -> All<T> {
    let (segments, minutes, analysis) = if entries.len() < 2 {
        let stop = start + SUNDAY_TO_SATURDAY;
        (
            entries.len(),
            minutes_between_days(&entries, start, stop, ts),
            None,
        )
    } else {
        let entry_minutes_by_day: Vec<Vec<i64>> = each_day_in_week(entries, start, ts)
            .filter(|(_, entries)| !entries.is_empty())
            .map(|(start, entries)| {
                let start = start.midnight().assume_offset(ts.local_offset());
                let stop = start + ONE_DAY;
                entries
                    .into_iter()
                    .map(|entry| entry.minutes_between(start, stop))
                    .collect()
            })
            .collect();
        let entry_minutes: Vec<i64> = entry_minutes_by_day
            .iter()
            .flat_map(|entries| entries.iter().copied())
            .collect();
        let segments = entry_minutes_by_day
            .iter()
            .fold(0, |sum, minutes| sum + minutes.len());
        let total_minutes = entry_minutes.iter().sum();
        let mean = total_minutes / segments as i64;
        let mut min = 10080;
        let mut max = 0;
        let mut sumsq = 0;
        for m in entry_minutes {
            if m < min {
                min = m
            }
            if m > max {
                max = m
            }
            let diff = m - mean;
            sumsq += diff * diff;
        }
        let stddev = sqrtint(sumsq / (segments as i64 - 1)) as i64;
        (
            segments,
            total_minutes,
            Some(AllAnalysis {
                min,
                mean,
                max,
                stddev,
                sparks: entry_minutes_by_day
                    .into_iter()
                    .map(|ms| ms.into_iter().map(|m| spark_for(m, max, sparks)).collect())
                    .collect(),
            }),
        )
    };
    All {
        start,
        minutes,
        segments,
        analysis,
    }
}

fn spark_for<T: Copy>(m: i64, max: i64, sparks: &[T]) -> T {
    let m = m as usize;
    let max = max as usize;
    match sparks.get(m * sparks.len() / max) {
        None => sparks[sparks.len() - 1],
        Some(s) => *s,
    }
}

fn minutes_between(entries: &[Entry], start: OffsetDateTime, stop: OffsetDateTime) -> i64 {
    entries
        .iter()
        .fold(0, |sum, entry| sum + entry.minutes_between(start, stop))
}

fn minutes_between_days<TS: TimeSource>(
    entries: &[Entry],
    start: Date,
    stop: Date,
    ts: &TS,
) -> i64 {
    minutes_between(
        entries,
        start.midnight().assume_offset(ts.local_offset()),
        stop.next_day().midnight().assume_offset(ts.local_offset()),
    )
}

fn sqrtint(n: i64) -> i64 {
    let mut i = 1;
    while i * i <= n {
        i += 1;
    }
    i - 1
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_entries;
    use crate::timesource::real_time::DefaultTimeSource;
    use time::date;

    type TestRes = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn test_typical() -> TestRes {
        let input = "2013-08-01 10:45,2013-08-01 11:15\n\
                     2013-08-02 10:15,2013-08-02 10:44\n\
                     2013-08-11 10:45,2013-08-11 11:46\n\
                     2013-08-22 10:45,2013-08-22 11:47\n\
                     2013-08-22 23:49,2013-08-23 00:02\n\
                     2013-08-31 10:45,2013-08-31 11:48\n\
                     2013-09-04 10:45,2013-09-04 11:04\n\
                     2013-09-04 11:04,2013-09-04 11:16\n\
                     2013-09-04 11:16,2013-09-04 11:26\n\
                     2013-09-05 11:26,2013-09-05 11:39\n\
                     2013-09-05 11:39,2013-09-05 11:49\n";
        let entries = parse_entries(input.as_bytes())?;
        let sparks = vec![0, 1, 2, 3, 4, 5, 6];
        let result = super::calc(entries, &sparks, &DefaultTimeSource);
        assert_eq!(
            result,
            vec![
                super::All {
                    start: date!(2013 - 07 - 28),
                    minutes: 59,
                    segments: 2,
                    analysis: Some(super::AllAnalysis {
                        min: 29,
                        mean: 29,
                        max: 30,
                        stddev: 1,
                        sparks: vec![vec![6], vec![6]]
                    })
                },
                super::All {
                    start: date!(2013 - 08 - 04),
                    minutes: 0,
                    segments: 0,
                    analysis: None
                },
                super::All {
                    start: date!(2013 - 08 - 11),
                    minutes: 61,
                    segments: 1,
                    analysis: None
                },
                super::All {
                    start: date!(2013 - 08 - 18),
                    minutes: 75,
                    segments: 3, // the one that spans midnight counts for each day.
                    analysis: Some(super::AllAnalysis {
                        min: 2,
                        mean: 25,
                        max: 62,
                        stddev: 32,
                        sparks: vec![vec![6, 1], vec![0]]
                    })
                },
                super::All {
                    start: date!(2013 - 08 - 25),
                    minutes: 63,
                    segments: 1,
                    analysis: None
                },
                super::All {
                    start: date!(2013 - 09 - 01),
                    minutes: 64,
                    segments: 5,
                    analysis: Some(super::AllAnalysis {
                        min: 10,
                        mean: 12,
                        max: 19,
                        stddev: 3,
                        sparks: vec![vec![6, 4, 3], vec![4, 3]]
                    })
                },
            ]
        );
        Ok(())
    }

    #[test]
    fn test_spark_for() {
        let sparks = ['a', 'b', 'c', 'd', 'e', 'f', 'g'];
        let check = |c, n| assert_eq!(c, super::spark_for(n, 322, &sparks), "{}/322", n);
        check('a', 0);
        check('a', 45);
        check('b', 46);
        check('b', 91);
        check('c', 92);
        check('c', 137);
        check('d', 138);
        check('d', 183);
        check('e', 184);
        check('e', 229);
        check('f', 230);
        check('f', 275);
        check('g', 276);
        check('g', 321);
        check('g', 322);
    }
}
