use crate::timesource::TimeSource;
use std::clone::Clone;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use time::{self, Date, OffsetDateTime, PrimitiveDateTime};

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub start: Time,
    pub stop: Option<Time>,
}

impl Entry {
    pub fn start<TS: TimeSource>(ts: &TS) -> Self {
        Self {
            start: Time::now(ts),
            stop: None,
        }
    }

    pub fn finish_if_not(self, stop: OffsetDateTime) -> Self {
        if self.is_finished() {
            self
        } else {
            Self {
                stop: Some(Time::at(stop)),
                ..self
            }
        }
    }
    pub fn finish<TS: TimeSource>(self, ts: &TS) -> Self {
        if self.is_finished() {
            panic!("finish called for a finished entry! {}", self);
        }
        Self {
            stop: Some(Time::now(ts)),
            ..self
        }
    }

    pub fn start_date(&self) -> Date {
        self.start.wrapped.date()
    }

    pub fn stop_date(&self) -> Option<Date> {
        self.stop.as_ref().map(|t| t.wrapped.date())
    }

    pub fn is_finished(&self) -> bool {
        self.stop.is_some()
    }

    pub fn includes_year(&self, year: i32) -> bool {
        if self.start.year() > year {
            return false;
        }
        if let Some(stop) = &self.stop {
            if stop.year() < year {
                return false;
            }
        }
        true
    }

    pub fn includes_year_month(&self, year: i32, month: u8) -> bool {
        match self.start.year() {
            y if y > year => return false,
            y if y == year => match self.start.month() {
                m if m > month => return false,
                _ => (),
            },
            _ => (),
        };
        if let Some(stop) = &self.stop {
            match stop.year() {
                y if y < year => return false,
                y if y == year => match stop.month() {
                    m if m < month => return false,
                    _ => (),
                },
                _ => (),
            };
        }
        true
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

    pub fn minutes<TS: TimeSource>(&self, ts: &TS) -> i64 {
        let duration = match &self.stop {
            None => ts.now() - self.start.wrapped,
            Some(t) => t.wrapped - self.start.wrapped,
        };
        duration.whole_minutes()
    }

    pub fn minutes_between(&self, from: OffsetDateTime, to: OffsetDateTime) -> i64 {
        if self.start.wrapped > to {
            return 0;
        }
        let start: OffsetDateTime = if self.start.wrapped > from {
            self.start.wrapped
        } else {
            from
        };
        let stop: OffsetDateTime = match &self.stop {
            None => to,
            Some(t) => {
                if t.wrapped < from {
                    return 0;
                } else if t.wrapped < to {
                    t.wrapped
                } else {
                    to
                }
            }
        };
        (stop - start).whole_minutes()
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.start)?;
        if let Some(stop) = &self.stop {
            writeln!(f, ",{}", stop)
        } else {
            writeln!(f)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Time {
    wrapped: OffsetDateTime,
    implied_tz: bool,
}

impl Time {
    pub fn now<TS: TimeSource>(ts: &TS) -> Self {
        Self {
            wrapped: ts.now(),
            implied_tz: false,
        }
    }

    pub fn at(wrapped: OffsetDateTime) -> Self {
        Self {
            wrapped,
            implied_tz: false,
        }
    }

    pub fn new<TS: TimeSource>(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        utc_offset: Option<i16>,
        ts: &TS,
    ) -> Result<Self, Box<dyn Error>> {
        let date = time::Date::try_from_ymd(year as i32, month, day)?;
        let time = time::Time::try_from_hms(hour, minute, 0)?;
        let offset = utc_offset.map(time::UtcOffset::minutes);
        Ok(Self::from_dto(date, time, offset, ts))
    }

    pub fn from_dto<TS: TimeSource>(
        date: time::Date,
        time: time::Time,
        offset: Option<time::UtcOffset>,
        ts: &TS,
    ) -> Self {
        let dt = PrimitiveDateTime::new(date, time);
        match offset {
            None => {
                let off = ts.local_offset();
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

    fn year(&self) -> i32 {
        self.wrapped.year()
    }

    fn month(&self) -> u8 {
        self.wrapped.month()
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
    use super::{Entry, Time};
    use crate::timesource::mock_time::mock_time;
    use crate::timesource::real_time::DefaultTimeSource;
    use time::{date, offset, time, PrimitiveDateTime};

    type TestRes = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn test_time_format_no_tz() -> TestRes {
        let time = Time::new(2020, 6, 20, 1, 7, None, &DefaultTimeSource)?;
        assert_eq!("2020-06-20 01:07", format!("{}", time));
        Ok(())
    }

    #[test]
    fn test_time_format_with_tz() -> TestRes {
        let time = Time::new(2020, 6, 20, 1, 7, Some(-123), &DefaultTimeSource)?;
        assert_eq!("2020-06-20 01:07 -0203", format!("{}", time));
        Ok(())
    }

    #[test]
    fn test_entry_format_with_start() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 1, 7, None, &DefaultTimeSource)?,
            stop: None,
        };
        assert_eq!("2020-06-20 01:07\n", format!("{}", entry));
        Ok(())
    }

    #[test]
    fn test_entry_format_with_start_and_stop() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 1, 7, None, &DefaultTimeSource)?,
            stop: Some(Time::new(2020, 6, 20, 1, 8, None, &DefaultTimeSource)?),
        };
        assert_eq!("2020-06-20 01:07,2020-06-20 01:08\n", format!("{}", entry));
        Ok(())
    }

    #[test]
    fn test_now() {
        let ts = mock_time(date!(2020 - 07 - 15), time!(11:23), offset!(+11:00));
        let time = Time::now(&ts);
        assert_eq!("2020-07-15 11:23 +1100", format!("{}", time));
    }

    #[test]
    fn test_minutes() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 1, 7, None, &DefaultTimeSource)?,
            stop: Some(Time::new(2020, 6, 20, 1, 8, None, &DefaultTimeSource)?),
        };
        assert_eq!(1, entry.minutes(&DefaultTimeSource));
        Ok(())
    }

    #[test]
    fn test_minutes_no_stop() -> TestRes {
        let ts = mock_time(date!(2020 - 06 - 20), time!(1:55), offset!(-02:00));
        let entry = Entry {
            start: Time::new(2020, 6, 20, 1, 7, Some(120), &ts)?,
            stop: None,
        };
        assert_eq!(4 * 60 + 48, entry.minutes(&ts));
        Ok(())
    }

    #[test]
    fn test_minutes_no_stop_no_tz() -> TestRes {
        let ts = mock_time(date!(2020 - 06 - 20), time!(1:55), offset!(+02:00));
        let entry = Entry {
            start: Time::new(2020, 6, 20, 1, 7, None, &ts)?,
            stop: None,
        };
        assert_eq!(48, entry.minutes(&ts));
        Ok(())
    }

    #[test]
    fn test_minutes_between() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 9, 0, Some(0), &DefaultTimeSource)?,
            stop: Some(Time::new(2020, 6, 20, 10, 0, Some(0), &DefaultTimeSource)?),
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(0:00)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 21), time!(0:00)).assume_utc();
        assert_eq!(60, entry.minutes_between(start, stop));
        Ok(())
    }

    #[test]
    fn test_minutes_between_after_stop() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 9, 0, Some(0), &DefaultTimeSource)?,
            stop: Some(Time::new(2020, 6, 20, 10, 0, Some(0), &DefaultTimeSource)?),
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 19), time!(0:00)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(0:00)).assume_utc();
        assert_eq!(0, entry.minutes_between(start, stop));
        Ok(())
    }

    #[test]
    fn test_minutes_between_before_start() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 9, 0, Some(0), &DefaultTimeSource)?,
            stop: Some(Time::new(2020, 6, 20, 10, 0, Some(0), &DefaultTimeSource)?),
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 21), time!(0:00)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 22), time!(0:00)).assume_utc();
        assert_eq!(0, entry.minutes_between(start, stop));
        Ok(())
    }

    #[test]
    fn test_minutes_between_entry_overlaps_start() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 9, 0, Some(0), &DefaultTimeSource)?,
            stop: Some(Time::new(2020, 6, 20, 10, 0, Some(0), &DefaultTimeSource)?),
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(0:00)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(9:30)).assume_utc();
        assert_eq!(30, entry.minutes_between(start, stop));
        Ok(())
    }

    #[test]
    fn test_minutes_between_entry_overlaps_stop() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 9, 0, Some(0), &DefaultTimeSource)?,
            stop: Some(Time::new(2020, 6, 20, 10, 0, Some(0), &DefaultTimeSource)?),
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(9:30)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 21), time!(0:00)).assume_utc();
        assert_eq!(30, entry.minutes_between(start, stop));
        Ok(())
    }

    #[test]
    fn test_minutes_between_entry_overlaps_start_and_stop() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 9, 0, Some(0), &DefaultTimeSource)?,
            stop: Some(Time::new(2020, 6, 20, 10, 0, Some(0), &DefaultTimeSource)?),
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(9:10)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(9:50)).assume_utc();
        assert_eq!(40, entry.minutes_between(start, stop));
        Ok(())
    }

    #[test]
    fn test_minutes_between_incomplete() -> TestRes {
        let entry = Entry {
            start: Time::new(2020, 6, 20, 9, 0, Some(0), &DefaultTimeSource)?,
            stop: None,
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(0:00)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 21), time!(0:00)).assume_utc();
        assert_eq!(15 * 60, entry.minutes_between(start, stop));
        Ok(())
    }
}
