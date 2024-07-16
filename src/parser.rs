use crate::entry::{into_time_entries, Entry, Time, TimeEntry, TZ};
use crate::timesource::TimeSource;
use std::error::Error;
use std::io::{self, BufRead, BufReader, Read, Write};

#[derive(Debug)]
struct ParseError {
    message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ParseError {}

pub fn parse_entries<R: Read, TS: TimeSource>(r: R, ts: &TS) -> Result<Vec<Entry>, Box<dyn Error>> {
    let r = BufReader::new(r);
    let mut parser = Parser::new(r, ts.local_offset());
    let mut res = Vec::with_capacity(1000);
    loop {
        match parser.parse_entry()? {
            None => break,
            Some(entry) => res.push(entry),
        }
    }
    Ok(res)
}

pub fn parse_time_entries<R: Read, TS: TimeSource>(
    r: R,
    ts: &TS,
) -> Result<Vec<TimeEntry>, Box<dyn Error>> {
    Ok(into_time_entries(parse_entries(r, ts)?))
}

pub fn parse_entry<R: BufRead, TS: TimeSource>(
    r: R,
    ts: &TS,
) -> Result<(Option<Entry>, R), Box<dyn Error>> {
    let mut parser = Parser::new(r, ts.local_offset());
    let entry = parser.parse_entry()?;
    Ok((entry, parser.reader))
}

pub fn write_entries(w: &mut impl Write, entries: &[Entry]) -> Result<(), Box<dyn Error>> {
    for entry in entries {
        write!(w, "{}", entry)?;
    }
    Ok(())
}

struct Parser<R: BufRead> {
    reader: R,
    default_tz: time::UtcOffset,
    line: usize,
    col: usize,
}

/// Morsel is either an annotation or the start of a time entry.
enum Morsel {
    None,
    Time(Time, bool),
    Note(String),
}

impl<R: BufRead> Parser<R> {
    fn new(r: R, default_tz: time::UtcOffset) -> Parser<R> {
        Parser {
            reader: r,
            default_tz,
            line: 1,
            col: 0,
        }
    }

    fn parse_entry(&mut self) -> Result<Option<Entry>, Box<dyn Error>> {
        match self.parse_morsel()? {
            Morsel::None => Ok(None),
            Morsel::Note(note) => Ok(Some(Entry::Note(note))),
            Morsel::Time(start, true) => Ok(Some(TimeEntry { start, stop: None }.into())),
            Morsel::Time(start, false) => match (self.parse_time(None))? {
                None => Ok(Some(TimeEntry { start, stop: None }.into())),
                Some((stop, _)) => Ok(Some(
                    TimeEntry {
                        start,
                        stop: Some(stop),
                    }
                    .into(),
                )),
            },
        }
    }

    fn parse_morsel(&mut self) -> Result<Morsel, Box<dyn Error>> {
        loop {
            match self.read()? {
                None => return Ok(Morsel::None),
                Some(b' ') | Some(b'\n') => (),
                Some(b'#') => return Ok(Morsel::Note(self.read_line()?)),
                Some(digit) => {
                    let digit = self.parse_digit(digit)?;
                    let year = 1000 * digit + self.read_number(100)?;
                    let (t, b) = self.parse_time(Some(year))?.unwrap();
                    return Ok(Morsel::Time(t, b));
                }
            }
        }
    }

    fn parse_time(&mut self, year: Option<u16>) -> Result<Option<(Time, bool)>, Box<dyn Error>> {
        let year = match year {
            Some(y) => Some(y),
            None => self.read_year()?,
        };
        match year {
            None => Ok(None),
            Some(year) => {
                self.read_expected(b'-')?;
                let month = self.read_number(10)? as u8;
                self.read_expected(b'-')?;
                let day = self.read_number(10)? as u8;
                self.read_expected(b' ')?;
                let hour = self.read_number(10)? as u8;
                self.read_expected(b':')?;
                let minute = self.read_number(10)? as u8;

                match self.read()? {
                    // EOF or EOL.
                    None | Some(b'\n') => Ok(Some((
                        Time::new(year, month, day, hour, minute, self.implied_tz())?,
                        true,
                    ))),
                    // End of current entry.
                    Some(b',') => Ok(Some((
                        Time::new(year, month, day, hour, minute, self.implied_tz())?,
                        false,
                    ))),
                    // TZ follows the space.
                    Some(b' ') => {
                        let sign: i16 = match self.read()? {
                            Some(b'-') => -1,
                            Some(b'+') => 1,
                            None => return Err(self.error("expected +/- but got EOF".to_string())),
                            Some(x) => {
                                return Err(
                                    self.error(format!("expected +/- but got '{}'", x as char))
                                )
                            }
                        };
                        let hr_off = self.read_number(10)? as i16;
                        let min_off = self.read_number(10)? as i16;

                        let total_min_off = sign * ((hr_off * 60) + min_off);
                        let tz = TZ::Known(time::UtcOffset::minutes(total_min_off));
                        let res = Time::new(year, month, day, hour, minute, tz)?;

                        match self.read()? {
                            // EOF or EOL.
                            None | Some(b'\n') => Ok(Some((res, true))),
                            // End of current entry.
                            Some(b',') => Ok(Some((res, false))),
                            // Anything else.
                            Some(x) => {
                                Err(self.error(format!("expected '\\n' but got '{}'", x as char)))
                            }
                        }
                    }
                    // Anything else.
                    Some(x) => Err(self.error(format!(
                        "expected newline, comma, or space, but got '{}'",
                        x as char
                    ))),
                }
            }
        }
    }

    fn implied_tz(&self) -> TZ {
        TZ::Implied(self.default_tz)
    }

    fn read_year(&mut self) -> Result<Option<u16>, Box<dyn Error>> {
        loop {
            match self.read()? {
                None => return Ok(None),
                Some(b' ') | Some(b'\n') => (),
                Some(digit) => {
                    let digit = self.parse_digit(digit)?;
                    let year = 1000 * digit + self.read_number(100)?;
                    return Ok(Some(year));
                }
            }
        }
    }

    fn read_line(&mut self) -> Result<String, Box<dyn Error>> {
        let mut s = String::new();
        self.reader.read_line(&mut s)?;
        if let Some('\n') = s.chars().last() {
            s.pop();
        }
        Ok(s)
    }

    fn read_expected(&mut self, expected: u8) -> Result<(), Box<dyn Error>> {
        match self.read()? {
            None => Err(self.error(format!("expected '{}' but got EOF", expected as char))),
            Some(x) => {
                if x == expected {
                    Ok(())
                } else {
                    Err(self.error(format!(
                        "expected '{}' but got '{}'",
                        expected as char, x as char
                    )))
                }
            }
        }
    }

    fn read_number(&mut self, scale: u16) -> Result<u16, Box<dyn Error>> {
        match self.read()? {
            None => Err(self.error("expected a digit but got EOF".to_string())),
            Some(digit) => {
                let digit = self.parse_digit(digit)?;
                match scale {
                    1 => Ok(digit),
                    _ => Ok(digit * scale + self.read_number(scale / 10)?),
                }
            }
        }
    }

    fn read(&mut self) -> io::Result<Option<u8>> {
        let mut buf = [0; 1];
        let len = self.reader.read(&mut buf)?;
        if len == 0 {
            return Ok(None);
        }
        let c = buf[0];
        if c == b'\n' {
            self.line += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }
        Ok(Some(c))
    }

    fn parse_digit(&self, digit: u8) -> Result<u16, Box<dyn Error>> {
        match digit {
            b'0' | b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' | b'9' => {
                Ok((digit - b'0') as u16)
            }
            x => Err(self.error(format!("expected a digit but got '{}'", x as char))),
        }
    }

    fn error(&self, message: String) -> Box<dyn Error> {
        let message = format!("line {}, col {}: {}", self.line, self.col, message);
        Box::new(ParseError { message })
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_entries, write_entries, Time, TimeEntry, TZ};
    use crate::{entry::Entry, timesource::real_time::DefaultTimeSource};

    type TestRes = Result<(), Box<dyn std::error::Error>>;

    fn mktime(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        offset: Option<i16>,
    ) -> Result<Time, Box<dyn std::error::Error>> {
        let tz = TZ::from(offset, &DefaultTimeSource);
        Time::new(year, month, day, hour, minute, tz)
    }

    #[test]
    fn test_empty() -> TestRes {
        let actual = parse_entries("".as_bytes(), &DefaultTimeSource)?;
        assert_eq!(Vec::<Entry>::new(), actual);
        Ok(())
    }

    #[test]
    fn test_start_no_tz() -> TestRes {
        let actual = parse_entries("2020-01-02 12:34\n".as_bytes(), &DefaultTimeSource)?;
        assert_eq!(
            vec![Entry::Time(TimeEntry {
                start: mktime(2020, 1, 2, 12, 34, None)?,
                stop: None,
            })],
            actual
        );
        Ok(())
    }

    #[test]
    fn test_start_with_neg_tz() -> TestRes {
        let actual = parse_entries("2020-01-02 12:34 -1001\n".as_bytes(), &DefaultTimeSource)?;
        assert_eq!(
            vec![Entry::Time(TimeEntry {
                start: mktime(2020, 1, 2, 12, 34, Some(-601))?,
                stop: None,
            })],
            actual
        );
        Ok(())
    }

    #[test]
    fn test_start_with_pos_tz_and_comma() -> TestRes {
        let actual = parse_entries("2020-01-02 12:34 +1001,\n".as_bytes(), &DefaultTimeSource)?;
        assert_eq!(
            vec![Entry::Time(TimeEntry {
                start: mktime(2020, 1, 2, 12, 34, Some(601))?,
                stop: None,
            })],
            actual
        );
        Ok(())
    }

    #[test]
    fn test_single_entry() -> TestRes {
        let actual = parse_entries(
            "2020-01-02 12:34 -0400,2020-01-02 13:34 -0400\n".as_bytes(),
            &DefaultTimeSource,
        )?;
        assert_eq!(
            vec![Entry::Time(TimeEntry {
                start: mktime(2020, 1, 2, 12, 34, Some(-240))?,
                stop: Some(mktime(2020, 1, 2, 13, 34, Some(-240))?),
            })],
            actual
        );
        Ok(())
    }

    #[test]
    fn test_single_entry_mixed_tz_and_space_between_times() -> TestRes {
        let actual = parse_entries(
            "2020-01-02 12:34 +1000,   2020-01-02 13:34 -0400\n".as_bytes(),
            &DefaultTimeSource,
        )?;
        assert_eq!(
            vec![Entry::Time(TimeEntry {
                start: mktime(2020, 1, 2, 12, 34, Some(600))?,
                stop: Some(mktime(2020, 1, 2, 13, 34, Some(-240))?),
            })],
            actual
        );
        Ok(())
    }

    #[test]
    fn test_several_entries() -> TestRes {
        let original = "2020-01-02 12:34 +1000,2020-01-02 13:34 -0400\n\
                        2020-01-03 09:00,2020-01-03 10:30\n\
                        2020-02-02 11:11,2020-02-02 12:12 -0400\n";
        let actual = parse_entries(original.as_bytes(), &DefaultTimeSource)?;
        assert_eq!(3, actual.len());

        // Verify that it round-trips.
        let mut output = Vec::new();
        write_entries(&mut output, &actual)?;
        assert_eq!(std::str::from_utf8(&output)?, original);
        Ok(())
    }

    #[test]
    fn test_several_entries_last_is_partial() -> TestRes {
        let original = "2020-01-02 12:34 +1000,2020-01-02 13:34 -0400\n\
                        2020-01-03 09:00,2020-01-03 10:30\n\
                        2020-02-02 11:11\n";
        let actual = parse_entries(original.as_bytes(), &DefaultTimeSource)?;
        assert_eq!(3, actual.len());
        assert_eq!(
            &TimeEntry {
                start: mktime(2020, 2, 2, 11, 11, None)?,
                stop: None,
            },
            actual[2].time()
        );

        // Verify that it round-trips.
        let mut output = Vec::new();
        write_entries(&mut output, &actual)?;
        assert_eq!(std::str::from_utf8(&output)?, original);
        Ok(())
    }

    #[test]
    fn test_annotations() -> TestRes {
        let original = "# first\n\
                        2020-01-02 12:34 -0400,2020-01-02 13:34 -0400\n\
                        # and another one \n\
                        2020-01-03 12:34 -0400,2020-01-03 14:00 -0400\n\
                        # lastly blah blah; no newline, oh the nerve!";
        let actual = parse_entries(original.as_bytes(), &DefaultTimeSource)?;
        assert_eq!(
            vec![
                Entry::note(" first"),
                Entry::Time(TimeEntry {
                    start: mktime(2020, 1, 2, 12, 34, Some(-240))?,
                    stop: Some(mktime(2020, 1, 2, 13, 34, Some(-240))?),
                }),
                Entry::note(" and another one "),
                Entry::Time(TimeEntry {
                    start: mktime(2020, 1, 3, 12, 34, Some(-240))?,
                    stop: Some(mktime(2020, 1, 3, 14, 0, Some(-240))?),
                }),
                Entry::note(" lastly blah blah; no newline, oh the nerve!"),
            ],
            actual
        );
        Ok(())
    }

    // TODO - tests for errors?
}
