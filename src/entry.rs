use crate::timesource::TimeSource;
use std::clone::Clone;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use time::{self, Date, OffsetDateTime, PrimitiveDateTime};

pub fn into_time_entries(entries: Vec<Entry>) -> Vec<TimeEntry> {
    entries
        .into_iter()
        .filter_map(|e| e.try_into_time())
        .collect()
}

#[derive(Debug, PartialEq)]
pub enum Entry {
    Time(TimeEntry),
    Note(String),
}

impl Entry {
    pub fn try_into_time(self) -> Option<TimeEntry> {
        match self {
            Self::Time(te) => Some(te),
            _ => None,
        }
    }

    pub fn into_time(self) -> TimeEntry {
        self.try_into_time().unwrap()
    }

    pub fn try_time(&self) -> Option<&TimeEntry> {
        match self {
            Self::Time(te) => Some(te),
            _ => None,
        }
    }

    pub fn time(&self) -> &TimeEntry {
        self.try_time().unwrap()
    }

    pub fn note(note: &str) -> Self {
        Self::Note(note.to_string())
    }
}

impl From<TimeEntry> for Entry {
    fn from(value: TimeEntry) -> Self {
        Self::Time(value)
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Time(te) => te.fmt(f),
            Self::Note(s) => write!(f, "# {s}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimeEntry {
    pub start: Time,
    pub stop: Option<Time>,
}

impl TimeEntry {
    pub fn start<TS: TimeSource>(ts: &TS) -> Self {
        Self {
            start: Time::at(ts.now()),
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
            stop: Some(Time::at(ts.now())),
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

    pub fn is_valid_after(&self, other: &Option<TimeEntry>) -> Result<(), String> {
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

    pub fn minutes_since_stop<TS: TimeSource>(&self, ts: &TS) -> Option<i64> {
        self.stop.as_ref()
            .and_then(|st| Some((ts.now() - st.wrapped).whole_minutes()))
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

impl Display for TimeEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.start)?;
        if let Some(stop) = &self.stop {
            writeln!(f, ",{}", stop)
        } else {
            writeln!(f)
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Time {
    wrapped: OffsetDateTime,
    implied_tz: bool,
}

pub enum TZ {
    Known(time::UtcOffset),
    Implied(time::UtcOffset),
}

impl TZ {
    pub fn from<TS: TimeSource>(utc_offset: Option<i16>, ts: &TS) -> Self {
        match utc_offset {
            None => Self::Implied(ts.local_offset()),
            Some(minutes) => Self::Known(time::UtcOffset::minutes(minutes)),
        }
    }
}

impl Time {
    pub fn at(wrapped: OffsetDateTime) -> Self {
        Self {
            wrapped,
            implied_tz: false,
        }
    }

    pub fn new(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        tz: TZ,
    ) -> Result<Self, Box<dyn Error>> {
        let date = time::Date::try_from_ymd(year as i32, month, day)?;
        let time = time::Time::try_from_hms(hour, minute, 0)?;
        Ok(Self::from_dto(date, time, tz))
    }

    fn from_dto(date: time::Date, time: time::Time, tz: TZ) -> Self {
        let dt = PrimitiveDateTime::new(date, time);
        match tz {
            TZ::Implied(off) => Self {
                wrapped: dt.assume_offset(off),
                implied_tz: true,
            },
            TZ::Known(off) => Self {
                wrapped: dt.assume_offset(off),
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

impl std::fmt::Debug for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Time{{{}}}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::{Time, TimeEntry, TZ};
    use crate::timesource::real_time::DefaultTimeSource;
    use crate::timesource::{mock_time::mock_time, TimeSource};
    use time::{date, offset, time, PrimitiveDateTime};

    type TestRes = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn test_time_format_no_tz() -> TestRes {
        let time = Time::new(2020, 6, 20, 1, 7, TZ::from(None, &DefaultTimeSource))?;
        assert_eq!("2020-06-20 01:07", format!("{}", time));
        Ok(())
    }

    #[test]
    fn test_time_format_with_tz() -> TestRes {
        let time = Time::new(2020, 6, 20, 1, 7, TZ::from(Some(-123), &DefaultTimeSource))?;
        assert_eq!("2020-06-20 01:07 -0203", format!("{}", time));
        Ok(())
    }

    #[test]
    fn test_entry_format_with_start() -> TestRes {
        let entry = TimeEntry {
            start: Time::new(2020, 6, 20, 1, 7, TZ::from(None, &DefaultTimeSource))?,
            stop: None,
        };
        assert_eq!("2020-06-20 01:07\n", format!("{}", entry));
        Ok(())
    }

    #[test]
    fn test_entry_format_with_start_and_stop() -> TestRes {
        let entry = TimeEntry {
            start: Time::new(2020, 6, 20, 1, 7, TZ::from(None, &DefaultTimeSource))?,
            stop: Some(Time::new(
                2020,
                6,
                20,
                1,
                8,
                TZ::from(None, &DefaultTimeSource),
            )?),
        };
        assert_eq!("2020-06-20 01:07,2020-06-20 01:08\n", format!("{}", entry));
        Ok(())
    }

    #[test]
    fn test_at() {
        let ts = mock_time(date!(2020 - 07 - 15), time!(11:23), offset!(+11:00));
        let time = Time::at(ts.now());
        assert_eq!("2020-07-15 11:23 +1100", format!("{}", time));
    }

    #[test]
    fn test_minutes() -> TestRes {
        let entry = TimeEntry {
            start: Time::new(2020, 6, 20, 1, 7, TZ::from(None, &DefaultTimeSource))?,
            stop: Some(Time::new(
                2020,
                6,
                20,
                1,
                8,
                TZ::from(None, &DefaultTimeSource),
            )?),
        };
        assert_eq!(1, entry.minutes(&DefaultTimeSource));
        Ok(())
    }

    #[test]
    fn test_minutes_no_stop() -> TestRes {
        let ts = mock_time(date!(2020 - 06 - 20), time!(1:55), offset!(-02:00));
        let entry = TimeEntry {
            start: Time::new(2020, 6, 20, 1, 7, TZ::from(Some(120), &ts))?,
            stop: None,
        };
        assert_eq!(4 * 60 + 48, entry.minutes(&ts));
        Ok(())
    }

    #[test]
    fn test_minutes_no_stop_no_tz() -> TestRes {
        let ts = mock_time(date!(2020 - 06 - 20), time!(1:55), offset!(+02:00));
        let entry = TimeEntry {
            start: Time::new(2020, 6, 20, 1, 7, TZ::from(None, &ts))?,
            stop: None,
        };
        assert_eq!(48, entry.minutes(&ts));
        Ok(())
    }

    #[test]
    fn test_minutes_between() -> TestRes {
        let entry = TimeEntry {
            start: Time::new(2020, 6, 20, 9, 0, TZ::from(Some(0), &DefaultTimeSource))?,
            stop: Some(Time::new(
                2020,
                6,
                20,
                10,
                0,
                TZ::from(Some(0), &DefaultTimeSource),
            )?),
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(0:00)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 21), time!(0:00)).assume_utc();
        assert_eq!(60, entry.minutes_between(start, stop));
        Ok(())
    }

    #[test]
    fn test_minutes_between_after_stop() -> TestRes {
        let entry = TimeEntry {
            start: Time::new(2020, 6, 20, 9, 0, TZ::from(Some(0), &DefaultTimeSource))?,
            stop: Some(Time::new(
                2020,
                6,
                20,
                10,
                0,
                TZ::from(Some(0), &DefaultTimeSource),
            )?),
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 19), time!(0:00)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(0:00)).assume_utc();
        assert_eq!(0, entry.minutes_between(start, stop));
        Ok(())
    }

    #[test]
    fn test_minutes_between_before_start() -> TestRes {
        let entry = TimeEntry {
            start: Time::new(2020, 6, 20, 9, 0, TZ::from(Some(0), &DefaultTimeSource))?,
            stop: Some(Time::new(
                2020,
                6,
                20,
                10,
                0,
                TZ::from(Some(0), &DefaultTimeSource),
            )?),
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 21), time!(0:00)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 22), time!(0:00)).assume_utc();
        assert_eq!(0, entry.minutes_between(start, stop));
        Ok(())
    }

    #[test]
    fn test_minutes_between_entry_overlaps_start() -> TestRes {
        let entry = TimeEntry {
            start: Time::new(2020, 6, 20, 9, 0, TZ::from(Some(0), &DefaultTimeSource))?,
            stop: Some(Time::new(
                2020,
                6,
                20,
                10,
                0,
                TZ::from(Some(0), &DefaultTimeSource),
            )?),
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(0:00)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(9:30)).assume_utc();
        assert_eq!(30, entry.minutes_between(start, stop));
        Ok(())
    }

    #[test]
    fn test_minutes_between_entry_overlaps_stop() -> TestRes {
        let entry = TimeEntry {
            start: Time::new(2020, 6, 20, 9, 0, TZ::from(Some(0), &DefaultTimeSource))?,
            stop: Some(Time::new(
                2020,
                6,
                20,
                10,
                0,
                TZ::from(Some(0), &DefaultTimeSource),
            )?),
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(9:30)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 21), time!(0:00)).assume_utc();
        assert_eq!(30, entry.minutes_between(start, stop));
        Ok(())
    }

    #[test]
    fn test_minutes_between_entry_overlaps_start_and_stop() -> TestRes {
        let entry = TimeEntry {
            start: Time::new(2020, 6, 20, 9, 0, TZ::from(Some(0), &DefaultTimeSource))?,
            stop: Some(Time::new(
                2020,
                6,
                20,
                10,
                0,
                TZ::from(Some(0), &DefaultTimeSource),
            )?),
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(9:10)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(9:50)).assume_utc();
        assert_eq!(40, entry.minutes_between(start, stop));
        Ok(())
    }

    #[test]
    fn test_minutes_between_incomplete() -> TestRes {
        let entry = TimeEntry {
            start: Time::new(2020, 6, 20, 9, 0, TZ::from(Some(0), &DefaultTimeSource))?,
            stop: None,
        };
        let start = PrimitiveDateTime::new(date!(2020 - 06 - 20), time!(0:00)).assume_utc();
        let stop = PrimitiveDateTime::new(date!(2020 - 06 - 21), time!(0:00)).assume_utc();
        assert_eq!(15 * 60, entry.minutes_between(start, stop));
        Ok(())
    }
}
