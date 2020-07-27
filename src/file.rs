use std::error::Error;
use std::io::{self, BufReader, Read};

#[derive(Debug, PartialEq)]
pub struct Entry {
    start: Time,
    stop: Option<Time>,
}

#[derive(Debug, PartialEq)]
pub struct Time {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    utc_offset: Option<i16>,
}

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

pub fn parse_entries(r: impl Read) -> Result<Vec<Entry>, Box<dyn Error>> {
    let mut parser = Parser::new(r);
    let mut res = vec![];
    loop {
        match parser.parse_entry()? {
            None => break,
            Some(entry) => res.push(entry),
        }
    }
    Ok(res)
}

struct Parser<R> {
    reader: BufReader<R>,
    line: usize,
    col: usize,
}

impl<R: Read> Parser<R> {
    fn new(r: R) -> Parser<R> {
        Parser {
            reader: BufReader::new(r),
            line: 1,
            col: 0,
        }
    }

    fn parse_entry(&mut self) -> Result<Option<Entry>, Box<dyn Error>> {
        match self.parse_time()? {
            None => Ok(None),
            Some((start, true)) => Ok(Some(Entry {
                start: start,
                stop: None,
            })),
            Some((start, false)) => match (self.parse_time())? {
                None => Ok(Some(Entry {
                    start: start,
                    stop: None,
                })),
                Some((stop, _)) => Ok(Some(Entry {
                    start: start,
                    stop: Some(stop),
                })),
            },
        }
    }

    fn parse_time(&mut self) -> Result<Option<(Time, bool)>, Box<dyn Error>> {
        match self.read_year()? {
            None => Ok(None),
            Some(year) => {
                let mut res = Time {
                    year,
                    month: 0,
                    day: 0,
                    hour: 0,
                    minute: 0,
                    utc_offset: None,
                };
                self.read_expected(b'-')?;
                res.month = self.read_number(10)? as u8;
                self.read_expected(b'-')?;
                res.day = self.read_number(10)? as u8;
                self.read_expected(b' ')?;
                res.hour = self.read_number(10)? as u8;
                self.read_expected(b':')?;
                res.minute = self.read_number(10)? as u8;
                match self.read()? {
                    None | Some(b'\n') => Ok(Some((res, true))),
                    Some(b',') => Ok(Some((res, false))),
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
                        res.utc_offset = Some(sign * ((hr_off * 60) + min_off));
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
        match self.read()? {
            None => Ok(None),
            Some(digit) => {
                let digit = self.parse_digit(digit)?;
                let year = 1000 * digit + self.read_number(100)?;
                Ok(Some(year))
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
            self.line = self.line + 1;
            self.col = 0;
        } else {
            self.col = self.col + 1;
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
    // variations:
    // - number of lines
    // - partial entry on last line vs complete entry
    // - time zone or not
    // - partial entry with/without comma

    use super::{parse_entries, Entry, Time};

    #[test]
    fn test_empty() {
        let actual = parse_entries("".as_bytes()).unwrap();
        assert_eq!(Vec::<Entry>::new(), actual);
    }

    #[test]
    fn test_start_no_tz() {
        let actual = parse_entries("2020-01-02 12:34\n".as_bytes()).unwrap();
        assert_eq!(
            vec![Entry {
                start: Time {
                    year: 2020,
                    month: 1,
                    day: 2,
                    hour: 12,
                    minute: 34,
                    utc_offset: None
                },
                stop: None,
            }],
            actual
        );
    }

    #[test]
    fn test_start_with_neg_tz() {
        let actual = parse_entries("2020-01-02 12:34 -1001\n".as_bytes()).unwrap();
        assert_eq!(
            vec![Entry {
                start: Time {
                    year: 2020,
                    month: 1,
                    day: 2,
                    hour: 12,
                    minute: 34,
                    utc_offset: Some(-601)
                },
                stop: None,
            }],
            actual
        );
    }

    #[test]
    fn test_start_with_pos_tz() {
        let actual = parse_entries("2020-01-02 12:34 +1001\n".as_bytes()).unwrap();
        assert_eq!(
            vec![Entry {
                start: Time {
                    year: 2020,
                    month: 1,
                    day: 2,
                    hour: 12,
                    minute: 34,
                    utc_offset: Some(601)
                },
                stop: None,
            }],
            actual
        );
    }

    #[test]
    fn test_single_entry() {
        let actual =
            parse_entries("2020-01-02 12:34 -0400,2020-01-02 13:34 -0400\n".as_bytes()).unwrap();
        assert_eq!(
            vec![Entry {
                start: Time {
                    year: 2020,
                    month: 1,
                    day: 2,
                    hour: 12,
                    minute: 34,
                    utc_offset: Some(-240)
                },
                stop: Some(Time {
                    year: 2020,
                    month: 1,
                    day: 2,
                    hour: 13,
                    minute: 34,
                    utc_offset: Some(-240)
                }),
            }],
            actual
        );
    }

    #[test]
    fn test_single_entry_mixed_tz() {
        let actual =
            parse_entries("2020-01-02 12:34 +1000,2020-01-02 13:34 -0400\n".as_bytes()).unwrap();
        assert_eq!(
            vec![Entry {
                start: Time {
                    year: 2020,
                    month: 1,
                    day: 2,
                    hour: 12,
                    minute: 34,
                    utc_offset: Some(600)
                },
                stop: Some(Time {
                    year: 2020,
                    month: 1,
                    day: 2,
                    hour: 13,
                    minute: 34,
                    utc_offset: Some(-240)
                }),
            }],
            actual
        );
    }

    #[test]
    fn test_several_entries() {
        let actual = parse_entries(
            "2020-01-02 12:34 +1000,2020-01-02 13:34 -0400\n\
                           2020-01-03 09:00, 2020-01-03 10:30\n\
                           2020-02-02 11:11, 2020-02-02 12:12 -0400\n"
                .as_bytes(),
        )
        .unwrap();
        assert_eq!(3, actual.len());
    }

    // test_several_entries_last_is_partial
}
