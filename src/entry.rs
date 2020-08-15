use std::error::Error;
use std::fmt::{self, Display, Formatter};
use time::{self, Date, OffsetDateTime, PrimitiveDateTime};

#[derive(Debug, PartialEq)]
pub struct Entry {
    pub start: Time,
    pub stop: Option<Time>,
}

#[cfg(not(test))]
pub mod real_time {
    use time::{OffsetDateTime, UtcOffset};

    // TODO - don't use this crate, it pulls in a bunch of dependencies.
    use cached::proc_macro::cached;

    pub fn now() -> OffsetDateTime {
        OffsetDateTime::now_local()
    }

    #[cached]
    pub fn local_offset() -> UtcOffset {
        UtcOffset::current_local_offset()
    }
}

// ok for test and prod.
fn explicit_offset(minutes: i16) -> time::UtcOffset {
    time::UtcOffset::minutes(minutes)
}

#[cfg(test)]
pub mod mock_time {
    // Adapted from https://blog.iany.me/2019/03/how-to-mock-time-in-rust-tests-and-cargo-gotchas-we-met/

    use std::cell::RefCell;
    use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

    thread_local! {
        static MOCK_TIME: RefCell<Option<OffsetDateTime>> = RefCell::new(None);
    }

    pub fn now() -> OffsetDateTime {
        MOCK_TIME.with(|cell| {
            cell.borrow()
                .as_ref()
                .cloned()
                .unwrap_or_else(OffsetDateTime::now_local)
        })
    }

    pub fn local_offset() -> UtcOffset {
        now().offset()
    }

    pub fn set_mock_time(date: Date, time: Time, offset: UtcOffset) {
        MOCK_TIME.with(|cell| {
            *cell.borrow_mut() = Some(PrimitiveDateTime::new(date, time).assume_offset(offset))
        });
    }

    pub fn clear_mock_time() {
        MOCK_TIME.with(|cell| *cell.borrow_mut() = None);
    }
}

#[cfg(test)]
use mock_time::{local_offset, now};
#[cfg(not(test))]
use real_time::{local_offset, now};

impl Entry {
    pub fn start() -> Self {
        Self {
            start: Time::now(),
            stop: None,
        }
    }

    pub fn finish(self) -> Self {
        if self.is_finished() {
            panic!("finish called for a finished entry! {}", self);
        }
        Self {
            start: self.start,
            stop: Some(Time::now()),
        }
    }

    pub fn start_date(&self) -> Date {
        self.start.wrapped.date()
    }

    pub fn is_finished(&self) -> bool {
        self.stop.is_some()
    }

    pub fn is_valid_after(&self, other: &Option<Entry>) -> Result<(), String> {
        if let Some(stop) = &self.stop {
            if self.start.wrapped > stop.wrapped {
                return Err(format!("{} should be before {}", self.start, stop));
            }
        }
        if let Some(other) = other {
            match &other.stop {
                None => return Err("previous entry is not complete!".to_string()),
                Some(stop) => {
                    if self.start.wrapped < stop.wrapped {
                        return Err(format!(
                            "{} starts before previous entry stops {}",
                            self.start, stop
                        ));
                    }
                }
            };
        }
        Ok(())
    }

    pub fn minutes(&self) -> i64 {
        let duration = match &self.stop {
            None => now() - self.start.wrapped,
            Some(t) => t.wrapped - self.start.wrapped,
        };
        duration.whole_minutes()
    }

    pub fn minutes_between(&self, from: &OffsetDateTime, to: &OffsetDateTime) -> i64 {
        if &self.start.wrapped > to {
            return 0;
        }
        let start: &OffsetDateTime = if &self.start.wrapped > from {
            &self.start.wrapped
        } else {
            from
        };
        let stop: &OffsetDateTime = match &self.stop {
            None => to,
            Some(t) => {
                if &t.wrapped < from {
                    return 0;
                } else if &t.wrapped < to {
                    &t.wrapped
                } else {
                    to
                }
            }
        };
        (*stop - *start).whole_minutes()
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.start)?;
        if let Some(stop) = &self.stop {
            write!(f, ",{}\n", stop)
        } else {
            write!(f, "\n")
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Time {
    wrapped: OffsetDateTime,
    implied_tz: bool,
}

impl Time {
    pub fn now() -> Self {
        Self {
            wrapped: now(),
            implied_tz: false,
        }
    }

    pub fn new(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        utc_offset: Option<i16>,
    ) -> Result<Self, Box<dyn Error>> {
        let date = time::Date::try_from_ymd(year as i32, month, day)?;
        let time = time::Time::try_from_hms(hour, minute, 0)?;
        let offset = utc_offset.and_then(|tz| Some(explicit_offset(tz)));
        Ok(Self::from_dto(date, time, offset))
    }

    pub fn from_dto(date: time::Date, time: time::Time, offset: Option<time::UtcOffset>) -> Self {
        let dt = PrimitiveDateTime::new(date, time);
        match offset {
            None => {
                let off = local_offset();
                Self {
                    wrapped: dt.assume_offset(off),
                    implied_tz: true,
                }
            }
            Some(tz) => Self {
                wrapped: dt.assume_offset(tz),
                implied_tz: false,
            },
        }
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let format = if self.implied_tz {
            "%Y-%m-%d %H:%M"
        } else {
            "%Y-%m-%d %H:%M %z"
        };
        write!(f, "{}", self.wrapped.format(format))
    }
}

#[cfg(test)]
mod tests {
    use super::{mock_time::*, Entry, Time};
    use time::{date, offset, time, PrimitiveDateTime};

    type TestRes = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn test_time_format_no_tz() -> TestRes {
        let time = Time::new(2020, 6, 20, 1, 7, None)?;
        assert_eq!("2020-06-20 01:07", format!("{}", time));
        Ok(())
    }

    #[test]
    fn test_time_format_with_tz() -> TestRes {
        let time = Time::new(2020, 6, 20, 1, 7, Some(-123))?;
        assert_eq!("2020-06-20 01:07 -0203", format!("{}", time));
        Ok(())
    }

    #[test]
    fn test_entry_format_with_start() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 1, 7, None)?,
            stop: None,
        };
        assert_eq!("2020-06-20 01:07\n", format!("{}", entry));
        Ok(())
    }

    #[test]
    fn test_entry_format_with_start_and_stop() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 1, 7, None)?,
            stop: Some(Time::new(2020, 6, 20, 1, 8, None)?),
        };
        assert_eq!("2020-06-20 01:07,2020-06-20 01:08\n", format!("{}", entry));
        Ok(())
    }

    #[test]
    fn test_now() {
        set_mock_time(date!(2020 - 07 - 15), time!(11:23), offset!(+11:00));
        let time = Time::now();
        assert_eq!("2020-07-15 11:23 +1100", format!("{}", time));
    }

    #[test]
    fn test_minutes() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 1, 7, None)?,
            stop: Some(Time::new(2020, 6, 20, 1, 8, None)?),
        };
        assert_eq!(1, entry.minutes());
        Ok(())
    }

    #[test]
    fn test_minutes_no_stop() -> TestRes {
        set_mock_time(date!(2020 - 06 - 20), time!(1:55), offset!(-02:00));
        let entry = Entry {
            start: Time::new(2020, 6, 20, 1, 7, Some(120))?,
            stop: None,
        };
        assert_eq!(4 * 60 + 48, entry.minutes());
        Ok(())
    }

    #[test]
    fn test_minutes_no_stop_no_tz() -> TestRes {
        set_mock_time(date!(2020 - 06 - 20), time!(1:55), offset!(+02:00));
        let entry = Entry {
            start: Time::new(2020, 6, 20, 1, 7, None)?,
            stop: None,
        };
        assert_eq!(48, entry.minutes());
        Ok(())
    }

    #[test]
    fn test_minutes_between() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 9, 0, Some(0))?,
            stop: Some(Time::new(2020, 6, 20, 10, 0, Some(0))?),
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(0:00)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 21), time!(0:00)).assume_utc();
        assert_eq!(60, entry.minutes_between(&start, &stop));
        Ok(())
    }

    #[test]
    fn test_minutes_between_after_stop() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 9, 0, Some(0))?,
            stop: Some(Time::new(2020, 6, 20, 10, 0, Some(0))?),
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 19), time!(0:00)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(0:00)).assume_utc();
        assert_eq!(0, entry.minutes_between(&start, &stop));
        Ok(())
    }

    #[test]
    fn test_minutes_between_before_start() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 9, 0, Some(0))?,
            stop: Some(Time::new(2020, 6, 20, 10, 0, Some(0))?),
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 21), time!(0:00)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 22), time!(0:00)).assume_utc();
        assert_eq!(0, entry.minutes_between(&start, &stop));
        Ok(())
    }

    #[test]
    fn test_minutes_between_entry_overlaps_start() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 9, 0, Some(0))?,
            stop: Some(Time::new(2020, 6, 20, 10, 0, Some(0))?),
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(0:00)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(9:30)).assume_utc();
        assert_eq!(30, entry.minutes_between(&start, &stop));
        Ok(())
    }

    #[test]
    fn test_minutes_between_entry_overlaps_stop() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 9, 0, Some(0))?,
            stop: Some(Time::new(2020, 6, 20, 10, 0, Some(0))?),
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(9:30)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 21), time!(0:00)).assume_utc();
        assert_eq!(30, entry.minutes_between(&start, &stop));
        Ok(())
    }

    #[test]
    fn test_minutes_between_entry_overlaps_start_and_stop() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 9, 0, Some(0))?,
            stop: Some(Time::new(2020, 6, 20, 10, 0, Some(0))?),
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(9:10)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(9:50)).assume_utc();
        assert_eq!(40, entry.minutes_between(&start, &stop));
        Ok(())
    }

    #[test]
    fn test_minutes_between_incomplete() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 9, 0, Some(0))?,
            stop: None,
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(0:00)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 21), time!(0:00)).assume_utc();
        assert_eq!(15 * 60, entry.minutes_between(&start, &stop));
        Ok(())
    }
}
