use crate::entry::{Entry, Time};
use crate::timesource::TimeSource;
use std::error::Error;
use std::io::{self, BufReader, Read, Write};

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

pub fn parse_entries<R: Read, TS: TimeSource + Clone>(
    r: R,
    ts: &TS,
) -> Result<Vec<Entry>, Box<dyn Error>> {
    let r = BufReader::new(r);
    let mut parser = Parser::new(r, ts.clone());
    let mut res = vec![];
    loop {
        match parser.parse_entry()? {
            None => break,
            Some(entry) => res.push(entry),
        }
    }
    Ok(res)
}

pub fn parse_entry<R: Read, TS: TimeSource + Clone>(
    r: R,
    ts: &TS,
) -> Result<(Option<Entry>, R), Box<dyn Error>> {
    let mut parser = Parser::new(r, ts.clone());
    let entry = parser.parse_entry()?;
    Ok((entry, parser.reader))
}

pub fn write_entries(w: &mut impl Write, entries: &[Entry]) -> Result<(), Box<dyn Error>> {
    for entry in entries {
        write!(w, "{}", entry)?;
    }
    Ok(())
}

struct Parser<R, TS> {
    reader: R,
    ts: TS,
    line: usize,
    col: usize,
}

impl<R: Read, TS: TimeSource + Clone> Parser<R, TS> {
    fn new(r: R, ts: TS) -> Parser<R, TS> {
        Parser {
            reader: r,
            ts,
            line: 1,
            col: 0,
        }
    }

    fn parse_entry(&mut self) -> Result<Option<Entry>, Box<dyn Error>> {
        match self.parse_time()? {
            None => Ok(None),
            Some((start, true)) => Ok(Some(Entry { start, stop: None })),
            Some((start, false)) => match (self.parse_time())? {
                None => Ok(Some(Entry { start, stop: None })),
                Some((stop, _)) => Ok(Some(Entry {
                    start,
                    stop: Some(stop),
                })),
            },
        }
    }

    fn parse_time(&mut self) -> Result<Option<(Time, bool)>, Box<dyn Error>> {
        match self.read_year()? {
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
                let ts = self.ts.clone();
                let res = |tz| Time::new(year, month, day, hour, minute, tz, &ts);
                match self.read()? {
                    None | Some(b'\n') => Ok(Some((res(None)?, true))),
                    Some(b',') => Ok(Some((res(None)?, false))),
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
                        let res = res(Some(sign * ((hr_off * 60) + min_off)))?;
                        match self.read()? {
                            None | Some(b'\n') => Ok(Some((res, true))),
                            Some(b',') => Ok(Some((res, false))),
                            Some(x) => {
                                Err(self.error(format!("expected '\\n' but got '{}'", x as char)))
                            }
                        }
                    }
                    Some(x) => Err(self.error(format!(
                        "expected newline, comma, or space, but got '{}'",
                        x as char
                    ))),
                }
            }
        }
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
    use super::{parse_entries, write_entries, Entry, Time};
    use crate::timesource::real_time::DefaultTimeSource;

    type TestRes = Result<(), Box<dyn std::error::Error>>;

    fn mktime(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        offset: Option<i16>,
    ) -> Result<Time, Box<dyn std::error::Error>> {
        Time::new(year, month, day, hour, minute, offset, &DefaultTimeSource)
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
            vec![Entry {
                start: mktime(2020, 1, 2, 12, 34, None)?,
                stop: None,
            }],
            actual
        );
        Ok(())
    }

    #[test]
    fn test_start_with_neg_tz() -> TestRes {
        let actual = parse_entries("2020-01-02 12:34 -1001\n".as_bytes(), &DefaultTimeSource)?;
        assert_eq!(
            vec![Entry {
                start: mktime(2020, 1, 2, 12, 34, Some(-601))?,
                stop: None,
            }],
            actual
        );
        Ok(())
    }

    #[test]
    fn test_start_with_pos_tz_and_comma() -> TestRes {
        let actual = parse_entries("2020-01-02 12:34 +1001,\n".as_bytes(), &DefaultTimeSource)?;
        assert_eq!(
            vec![Entry {
                start: mktime(2020, 1, 2, 12, 34, Some(601))?,
                stop: None,
            }],
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
            vec![Entry {
                start: mktime(2020, 1, 2, 12, 34, Some(-240))?,
                stop: Some(mktime(2020, 1, 2, 13, 34, Some(-240))?),
            }],
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
            vec![Entry {
                start: mktime(2020, 1, 2, 12, 34, Some(600))?,
                stop: Some(mktime(2020, 1, 2, 13, 34, Some(-240))?),
            }],
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
            Entry {
                start: mktime(2020, 2, 2, 11, 11, None)?,
                stop: None,
            },
            actual[2]
        );

        // Verify that it round-trips.
        let mut output = Vec::new();
        write_entries(&mut output, &actual)?;
        assert_eq!(std::str::from_utf8(&output)?, original);
        Ok(())
    }

    // TODO - tests for errors?
}
