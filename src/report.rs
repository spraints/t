use crate::entry::{local_offset, Entry};
use crate::iter::{each_day, each_week};
use time::{Date, Duration, OffsetDateTime};

#[derive(Debug, PartialEq)]
struct All<T: PartialEq> {
    start: Date,
    minutes: i64,
    segments: usize,
    analysis: Option<AllAnalysis<T>>,
}

#[derive(Debug, PartialEq)]
struct AllAnalysis<T: PartialEq> {
    min: i64,
    max: i64,
    mean: i64,
    stddev: i64,
    sparks: Vec<Vec<T>>,
}

fn calc_all<T: PartialEq + Copy>(entries: Vec<Entry>, sparks: &Vec<T>) -> Vec<All<T>> {
    each_week(entries)
        .map(|(start, entries)| calc_all_week(start, entries, sparks))
        .collect()
}

const ONE_DAY: Duration = Duration::days(1);
const SUNDAY_TO_SATURDAY: Duration = Duration::days(6);

fn calc_all_week<T: PartialEq + Copy>(start: Date, entries: Vec<Entry>, sparks: &Vec<T>) -> All<T> {
    let segments = entries.len();
    let (minutes, analysis) = if segments < 2 {
        let stop = start + SUNDAY_TO_SATURDAY;
        (minutes_between_days(&entries, start, stop), None)
    } else {
        let entry_minutes_by_day: Vec<Vec<i64>> = each_day(entries)
            .filter(|(_, entries)| !entries.is_empty())
            .map(|(start, entries)| {
                let start = start.midnight().assume_offset(local_offset());
                let stop = start + ONE_DAY;
                entries
                    .into_iter()
                    .map(|entry| entry.minutes_between(start, stop))
                    .collect()
            })
            .collect();
        let entry_minutes: Vec<i64> = entry_minutes_by_day
            .iter()
            .flat_map(|entries| entries.iter().map(|e| *e))
            .collect();
        let total_minutes = entry_minutes.iter().fold(0, |sum, minutes| sum + minutes);
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

fn spark_for<T: Copy>(m: i64, max: i64, sparks: &Vec<T>) -> T {
    let binsize = 1 + (max - 1) / sparks.len() as i64;
    let i = m / binsize;
    sparks[i as usize]
}

fn minutes_between(entries: &Vec<Entry>, start: OffsetDateTime, stop: OffsetDateTime) -> i64 {
    entries
        .iter()
        .fold(0, |sum, entry| sum + entry.minutes_between(start, stop))
}

fn minutes_between_days(entries: &Vec<Entry>, start: Date, stop: Date) -> i64 {
    minutes_between(
        entries,
        start.midnight().assume_offset(local_offset()),
        stop.next_day().midnight().assume_offset(local_offset()),
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
    use time::date;

    type TestRes = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn test_typical() -> TestRes {
        let input = "2013-08-01 10:45,2013-08-01 11:15\n\
                     2013-08-02 10:15,2013-08-02 10:44\n\
                     2013-08-11 10:45,2013-08-11 11:46\n\
                     2013-08-22 10:45,2013-08-22 11:47\n\
                     2013-08-31 10:45,2013-08-31 11:48\n\
                     2013-09-04 10:45,2013-09-04 11:04\n\
                     2013-09-04 11:04,2013-09-04 11:16\n\
                     2013-09-04 11:16,2013-09-04 11:26\n\
                     2013-09-05 11:26,2013-09-05 11:39\n\
                     2013-09-05 11:39,2013-09-05 11:49\n";
        let entries = parse_entries(input.as_bytes())?;
        let sparks = vec![0, 1, 2, 3, 4, 5, 6];
        let result = super::calc_all(entries, &sparks);
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
                        sparks: vec![vec![1], vec![6], vec![6]]
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
                    segments: 0,
                    analysis: None
                },
                super::All {
                    start: date!(2013 - 08 - 18),
                    minutes: 62,
                    segments: 0,
                    analysis: None
                },
                super::All {
                    start: date!(2013 - 08 - 25),
                    minutes: 63,
                    segments: 0,
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
                        sparks: vec![vec![3], vec![6, 4, 3], vec![4, 3]]
                    })
                },
            ]
        );
        Ok(())
    }
}
