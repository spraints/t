use time::{Date, Duration, OffsetDateTime};

use crate::entry::TimeEntry;
use crate::iter::each_week;
use crate::timesource::TimeSource;

#[derive(Debug)]
pub struct Punchcard<T> {
    pub start: Date,
    pub minutes: i64,
    pub punches: Vec<T>,
}

pub fn calc<T: Copy, TS: TimeSource>(
    entries: Vec<TimeEntry>,
    display_width: usize,
    empty_spark: T,
    sparks: &[T],
    ts: &TS,
) -> Vec<Punchcard<T>> {
    let spark_width = Duration::days(7) / (display_width as i32);
    let spark_increment = spark_width.whole_minutes() / (sparks.len() as i64);
    let sizes = Sizes {
        display_width,
        spark_width,
        spark_increment,
    };
    each_week(entries, ts)
        .map(|(start, entries)| punchit(start, entries, &sizes, empty_spark, sparks, ts))
        .collect()
}

struct Sizes {
    display_width: usize,
    spark_width: Duration,
    spark_increment: i64,
}

const SUNDAY_TO_SATURDAY: Duration = Duration::days(6);

fn punchit<T: Copy, TS: TimeSource>(
    start: Date,
    entries: Vec<TimeEntry>,
    sizes: &Sizes,
    empty_spark: T,
    sparks: &[T],
    ts: &TS,
) -> Punchcard<T> {
    let stop = start + SUNDAY_TO_SATURDAY;
    let minutes = minutes_between_days(&entries, start, stop, ts);
    let start_time = start.midnight().assume_offset(ts.local_offset());
    let max_spark: T = sparks[sparks.len() - 1];
    let punches = (0..sizes.display_width)
        .map(|i| {
            let slice_start = start_time + sizes.spark_width * (i as i32);
            let slice_stop = slice_start + sizes.spark_width;
            let slice_minutes = minutes_between(&entries, slice_start, slice_stop);
            if slice_minutes == 0 {
                empty_spark
            } else {
                let spark_index = slice_minutes / sizes.spark_increment;
                sparks
                    .get(spark_index as usize)
                    .copied()
                    .unwrap_or(max_spark)
            }
        })
        .collect();
    Punchcard {
        start,
        minutes,
        punches,
    }
}

fn minutes_between_days<TS: TimeSource>(
    entries: &[TimeEntry],
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

fn minutes_between(entries: &[TimeEntry], start: OffsetDateTime, stop: OffsetDateTime) -> i64 {
    entries
        .iter()
        .fold(0, |sum, entry| sum + entry.minutes_between(start, stop))
}
