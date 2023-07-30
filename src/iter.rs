use crate::entry::TimeEntry;
use crate::timesource::TimeSource;
use time::{Date, Duration, OffsetDateTime, Weekday::*};

pub fn each_week<TS: TimeSource>(entries: Vec<TimeEntry>, ts: &TS) -> DaysIterator {
    DaysIterator {
        entries,
        days: 7,
        last_date: None,
        next_index: 0,
        now: ts.now(),
    }
}

fn each_day<TS: TimeSource>(entries: Vec<TimeEntry>, ts: &TS) -> DaysIterator {
    DaysIterator {
        entries,
        days: 1,
        last_date: None,
        next_index: 0,
        now: ts.now(),
    }
}

pub fn each_day_in_week<TS: TimeSource>(
    entries: Vec<TimeEntry>,
    week_start: Date,
    ts: &TS,
) -> WeekOfDaysIterator {
    WeekOfDaysIterator {
        days_iter: each_day(entries, ts),
        week_start,
    }
}

pub struct DaysIterator {
    entries: Vec<TimeEntry>,
    days: u8,
    last_date: Option<Date>,
    next_index: usize,
    now: OffsetDateTime,
}

impl Iterator for DaysIterator {
    type Item = (Date, Vec<TimeEntry>);

    fn next(&mut self) -> std::option::Option<Self::Item> {
        if self.next_index >= self.entries.len() {
            None
        } else {
            let date = match self.last_date {
                None => self.get_first_date(),
                Some(d) => d + self.span(),
            };
            let next_date = date + self.span();
            self.last_date = Some(date);
            let mut entries = vec![];
            for entry in self.entries.iter().skip(self.next_index) {
                if entry.start_date() >= next_date {
                    break;
                } else {
                    entries.push(entry.clone().finish_if_not(self.now));
                    let st = match entry.stop_date() {
                        None => self.now.date(),
                        Some(d) => d,
                    };
                    if st >= next_date {
                        break;
                    }
                    self.next_index += 1;
                }
            }
            Some((date, entries))
        }
    }
}

impl DaysIterator {
    fn span(&self) -> Duration {
        Duration::days(self.days as i64)
    }

    fn get_first_date(&self) -> Date {
        match self.days {
            1 => self.entries[0].start_date(),
            7 => {
                let date = self.entries[0].start_date();
                match date.weekday() {
                    Sunday => date,
                    Monday => date - Duration::days(1),
                    Tuesday => date - Duration::days(2),
                    Wednesday => date - Duration::days(3),
                    Thursday => date - Duration::days(4),
                    Friday => date - Duration::days(5),
                    Saturday => date - Duration::days(6),
                }
            }
            x => panic!("Unable to iterate with span of {} days!", x),
        }
    }
}

const SUNDAY_TO_SATURDAY: Duration = Duration::days(6);

pub struct WeekOfDaysIterator {
    days_iter: DaysIterator,
    week_start: Date,
}

impl Iterator for WeekOfDaysIterator {
    type Item = (Date, Vec<TimeEntry>);

    fn next(&mut self) -> std::option::Option<Self::Item> {
        match self.days_iter.next() {
            None => None,
            Some((d, e)) => {
                if d < self.week_start || d > (self.week_start + SUNDAY_TO_SATURDAY) {
                    self.next()
                } else {
                    Some((d, e))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_time_entries;
    use crate::timesource::mock_time::mock_time;
    use crate::timesource::real_time::DefaultTimeSource;
    use pretty_assertions::assert_eq;
    use time::{date, offset, time};

    type TestRes = Result<(), Box<dyn std::error::Error>>;

    #[test]
    pub fn test_each_week_empty() {
        let mut i = super::each_week(vec![], &DefaultTimeSource);
        assert_eq!(None, i.next());
    }

    #[test]
    pub fn test_each_day_empty() {
        let mut i = super::each_day(vec![], &DefaultTimeSource);
        assert_eq!(None, i.next());
    }

    #[test]
    pub fn test_each_week_one_week() -> TestRes {
        let entries = parse_time_entries(
            "2020-08-02 10:10,2020-08-02 11:10\n\
             2020-08-08 10:10,2020-08-08 11:10\n"
                .as_bytes(),
            &DefaultTimeSource,
        )?;
        let mut i = super::each_week(entries.clone(), &DefaultTimeSource);
        assert_eq!(Some((date!(2020 - 08 - 02), entries)), i.next());
        assert_eq!(None, i.next());
        Ok(())
    }

    #[test]
    pub fn test_each_day_one_day() -> TestRes {
        let entries = parse_time_entries(
            "2020-08-02 10:10,2020-08-02 11:10\n\
             2020-08-02 10:10,2020-08-02 11:10\n"
                .as_bytes(),
            &DefaultTimeSource,
        )?;
        let mut i = super::each_day(entries.clone(), &DefaultTimeSource);
        assert_eq!(Some((date!(2020 - 08 - 02), entries)), i.next());
        assert_eq!(None, i.next());
        Ok(())
    }

    #[test]
    pub fn test_each_week_two_weeks() -> TestRes {
        let mut entries = parse_time_entries(
            "2020-08-02 10:10,2020-08-02 11:10\n\
             2020-09-02 10:10,2020-09-02 11:10\n"
                .as_bytes(),
            &DefaultTimeSource,
        )?;
        let mut i = super::each_week(entries.clone(), &DefaultTimeSource);
        assert_eq!(
            Some((date!(2020 - 08 - 02), entries.drain(..1).collect())),
            i.next()
        );
        assert_eq!(Some((date!(2020 - 08 - 09), vec![])), i.next());
        assert_eq!(Some((date!(2020 - 08 - 16), vec![])), i.next());
        assert_eq!(Some((date!(2020 - 08 - 23), vec![])), i.next());
        assert_eq!(Some((date!(2020 - 08 - 30), entries)), i.next());
        assert_eq!(None, i.next());
        Ok(())
    }

    #[test]
    pub fn test_each_day_two_days() -> TestRes {
        let mut entries = parse_time_entries(
            "2020-08-02 10:10,2020-08-02 11:10\n\
             2020-08-05 10:10,2020-08-05 11:10\n"
                .as_bytes(),
            &DefaultTimeSource,
        )?;
        let mut i = super::each_day(entries.clone(), &DefaultTimeSource);
        assert_eq!(
            Some((date!(2020 - 08 - 02), entries.drain(..1).collect())),
            i.next()
        );
        assert_eq!(Some((date!(2020 - 08 - 03), vec![])), i.next());
        assert_eq!(Some((date!(2020 - 08 - 04), vec![])), i.next());
        assert_eq!(Some((date!(2020 - 08 - 05), entries)), i.next());
        assert_eq!(None, i.next());
        Ok(())
    }

    #[test]
    pub fn test_each_week_entry_spans_weeks() -> TestRes {
        let entries = parse_time_entries(
            "2020-08-02 10:10,2020-08-02 11:10\n\
             2020-08-08 10:10,2020-08-09 11:10\n\
             2020-09-02 10:10,2020-09-02 11:10\n"
                .as_bytes(),
            &DefaultTimeSource,
        )?;
        let mut i = super::each_week(entries.clone(), &DefaultTimeSource);
        assert_eq!(
            Some((
                date!(2020 - 08 - 02),
                vec![entries[0].clone(), entries[1].clone()]
            )),
            i.next()
        );
        assert_eq!(
            Some((date!(2020 - 08 - 09), vec![entries[1].clone()])),
            i.next()
        );
        assert_eq!(Some((date!(2020 - 08 - 16), vec![])), i.next());
        assert_eq!(Some((date!(2020 - 08 - 23), vec![])), i.next());
        assert_eq!(
            Some((date!(2020 - 08 - 30), vec![entries[2].clone()])),
            i.next()
        );
        assert_eq!(None, i.next());
        Ok(())
    }

    #[test]
    pub fn test_each_day_entry_spans_days() -> TestRes {
        let entries = parse_time_entries(
            "2020-08-02 10:10,2020-08-02 11:10\n\
             2020-08-02 12:10,2020-08-03 11:10\n\
             2020-08-05 10:10,2020-08-05 11:10\n"
                .as_bytes(),
            &DefaultTimeSource,
        )?;
        let mut i = super::each_day(entries.clone(), &DefaultTimeSource);
        assert_eq!(
            Some((
                date!(2020 - 08 - 02),
                vec![entries[0].clone(), entries[1].clone()]
            )),
            i.next()
        );
        assert_eq!(
            Some((date!(2020 - 08 - 03), vec![entries[1].clone()])),
            i.next()
        );
        assert_eq!(Some((date!(2020 - 08 - 04), vec![])), i.next());
        assert_eq!(
            Some((date!(2020 - 08 - 05), vec![entries[2].clone()])),
            i.next()
        );
        assert_eq!(None, i.next());
        Ok(())
    }

    #[test]
    pub fn test_each_week_first_entry_not_sunday() -> TestRes {
        let entries = parse_time_entries(
            "2020-08-08 10:10,2020-08-08 11:10\n".as_bytes(),
            &DefaultTimeSource,
        )?;
        let mut i = super::each_week(entries.clone(), &DefaultTimeSource);
        assert_eq!(Some((date!(2020 - 08 - 02), entries)), i.next());
        assert_eq!(None, i.next());
        Ok(())
    }

    #[test]
    fn test_each_week_current_entry_in_progress() -> TestRes {
        let ts = mock_time(date!(2020 - 01 - 15), time!(11:00), offset!(-04:00));
        let entries = parse_time_entries("2020-01-15 10:10 -0400".as_bytes(), &ts)?;
        let expected_entries = parse_time_entries(
            "2020-01-15 10:10 -0400,2020-01-15 11:00 -0400".as_bytes(),
            &ts,
        )?;
        let mut i = super::each_week(entries.clone(), &ts);
        assert_eq!(Some((date!(2020 - 01 - 12), expected_entries)), i.next());
        assert_eq!(None, i.next());
        Ok(())
    }

    #[test]
    fn test_each_day_in_week() -> TestRes {
        let entries = parse_time_entries(
            "2020-08-01 10:10,2020-08-01 11:10\n\
             2020-08-02 10:10,2020-08-03 11:10\n\
             2020-08-04 12:10,2020-08-04 11:10\n\
             2020-08-05 10:10,2020-08-05 11:10\n\
             2020-08-08 10:10,2020-08-11 11:10\n"
                .as_bytes(),
            &DefaultTimeSource,
        )?;
        let mut i =
            super::each_day_in_week(entries.clone(), date!(2020 - 08 - 03), &DefaultTimeSource);
        assert_eq!(
            Some((date!(2020 - 08 - 03), vec![entries[1].clone()])),
            i.next()
        );
        assert_eq!(
            Some((date!(2020 - 08 - 04), vec![entries[2].clone()])),
            i.next()
        );
        assert_eq!(
            Some((date!(2020 - 08 - 05), vec![entries[3].clone()])),
            i.next()
        );
        assert_eq!(Some((date!(2020 - 08 - 06), vec![])), i.next());
        assert_eq!(Some((date!(2020 - 08 - 07), vec![])), i.next());
        assert_eq!(
            Some((date!(2020 - 08 - 08), vec![entries[4].clone()])),
            i.next()
        );
        assert_eq!(
            Some((date!(2020 - 08 - 09), vec![entries[4].clone()])),
            i.next()
        );
        Ok(())
    }
}
