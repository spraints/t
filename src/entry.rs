use std::error::Error;
use std::fmt::{self, Display, Formatter};
use time::{self, Date, OffsetDateTime, PrimitiveDateTime};

#[derive(Debug, PartialEq)]
pub struct Entry {
    pub start: Time,
    pub stop: Option<Time>,
}

impl Entry {
    pub fn start() -> Self {
        Entry {
            start: Time::now(),
            stop: None,
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
            None => OffsetDateTime::now_local() - self.start.wrapped,
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
            wrapped: OffsetDateTime::now_local(),
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
        let dt = PrimitiveDateTime::new(date, time);
        match utc_offset {
            None => {
                let off = unsafe { local_offset() };
                Ok(Time {
                    wrapped: dt.assume_offset(off),
                    implied_tz: true,
                })
            }
            Some(tz) => Ok(Time {
                wrapped: dt.assume_offset(explicit_offset(tz)),
                implied_tz: false,
            }),
        }
    }
}

unsafe fn local_offset() -> time::UtcOffset {
    static mut LOCAL_OFFSET: Option<time::UtcOffset> = None;
    match LOCAL_OFFSET {
        None => {
            let res = time::UtcOffset::current_local_offset();
            LOCAL_OFFSET = Some(res);
            res
        }
        Some(res) => res,
    }
}

fn explicit_offset(minutes: i16) -> time::UtcOffset {
    time::UtcOffset::minutes(minutes)
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
    use super::{Entry, Time};
    use time::{date, time, PrimitiveDateTime};

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
        let time = Time::now();
        assert!(time.wrapped.timestamp() > 0);
        assert_eq!(false, time.implied_tz);
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
        let entry = Entry {
            start: Time::new(2020, 6, 20, 1, 7, None)?,
            stop: None,
        };
        assert!(entry.minutes() > 0);
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
