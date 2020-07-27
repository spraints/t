use std::fmt::{self, Formatter, Display};

#[derive(Debug, PartialEq)]
pub struct Entry {
    pub start: Time,
    pub stop: Option<Time>,
}

impl Entry {
    fn minutes(&self) -> u16 {
        panic!("TODO: return number of minutes represented by this Entry");
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
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub utc_offset: Option<i16>,
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.utc_offset {
            None => write!(f, "{:04}-{:02}-{:02} {:02}:{:02}", self.year, self.month, self.day, self.hour, self.minute),
            Some(off) => {
                let (sign, off) = if off < 0 { ("-", -off) } else { ("+", off) };
                let hr_off = off / 60;
                let min_off = off % 60;
            write!(f, "{:04}-{:02}-{:02} {:02}:{:02} {}{:02}{:02}", self.year, self.month, self.day, self.hour, self.minute, sign, hr_off,min_off)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Entry, Time};

    #[test]
    fn test_time_format_no_tz() {
        let time = Time{year: 2020, month: 6, day: 20, hour: 1, minute:7, utc_offset: None};
        assert_eq!("2020-06-20 01:07", format!("{}", time));
    }

    #[test]
    fn test_time_format_with_tz() {
        let time = Time{year: 2020, month: 6, day: 20, hour: 1, minute:7, utc_offset: Some(-123)};
        assert_eq!("2020-06-20 01:07 -0203", format!("{}", time));
    }

    #[test]
    fn test_entry_format_with_start() {
        let entry = Entry{
            start:Time{year: 2020, month: 6, day: 20, hour: 1, minute:7, utc_offset: None},
            stop: None,
        };
        assert_eq!("2020-06-20 01:07\n", format!("{}", entry));
    }

    #[test]
    fn test_entry_format_with_start_and_stop() {
        let entry = Entry{
            start:Time{year: 2020, month: 6, day: 20, hour: 1, minute:7, utc_offset: None},
            stop: Some(Time{year: 2020, month: 6, day: 20, hour: 1, minute:8, utc_offset: None}),
        };
        assert_eq!("2020-06-20 01:07,2020-06-20 01:08\n", format!("{}", entry));
    }

    // #[test]
    // fn test_now() {
    //     panic!("TODO: Entry::now()");
    // }

    // #[test]
    // fn test_minutes() {
    //     panic!("TODO: Entry{...}.minutes");
    // }

    // #[test]
    // fn test_minutes_on_day() {
    //     panic!("TODO: Entry{...}.minutes_on_day(2020, 1, 1)");
    // }

    // #[test]
    // fn test_minutes_partial() {
    //     panic!("TODO: Entry{...}.minutes");
    // }
}
