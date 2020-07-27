use std::io::{self, BufReader, Read};
use std::error::Error;

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

impl Error for ParseError {
}

pub fn parse_entries(r: impl Read) -> Result<Vec<Entry>, Box<dyn Error>> {
    let mut r = BufReader::new(r);
    let mut res = vec![];
    loop {
        match parse_entry(&mut r)? {
            None => break,
            Some(entry) => res.push(entry),
        }
    }
    Ok(res)
}

fn parse_entry(r: &mut impl Read) -> Result<Option<Entry>, Box<dyn Error>> {
    match parse_time(r)? {
        None => Ok(None),
        Some((start, true)) => Ok(Some(Entry{start: start, stop: None})),
        Some((start, false)) => match(parse_time(r))? {
            None => Ok(Some(Entry{start: start, stop: None})),
            Some((stop, _)) => Ok(Some(Entry{start: start, stop: Some(stop)})),
        }
    }
}

fn parse_time(r: &mut impl Read) -> Result<Option<(Time, bool)>, Box<dyn Error>> {
    match read_year(r)? {
        None => Ok(None),
        Some(year) => {
            let mut res = Time{year, month: 0, day: 0, hour: 0, minute: 0, utc_offset: None};
            read_expected(r, b'-')?;
            res.month = read_number(r, 10)? as u8;
            read_expected(r, b'-')?;
            res.day = read_number(r, 10)? as u8;
            read_expected(r, b' ')?;
            res.hour = read_number(r, 10)? as u8;
            read_expected(r, b':')?;
            res.minute = read_number(r, 10)? as u8;
            match read(r)? {
                None | Some(b'\n') => Ok(Some((res, true))),
                Some(b',') => Ok(Some((res, false))),
                Some(b' ') => {
                    let sign: i16 = match read(r)? {
                        Some(b'-') => -1,
                        Some(b'+') => 1,
                        x => return Err(Box::new(ParseError{message: format!("expected +/- but got {:?}", x)})),
                    };
                    let hr_off = read_number(r, 10)? as i16;
                    let min_off = read_number(r, 10)? as i16;
                    res.utc_offset = Some(sign * ((hr_off * 60)  + min_off));
                    match read(r)? {
                        None => Ok(Some((res, true))),
                        Some(b'\n') => Ok(Some((res, true))),
                        Some(x) => Err(Box::new(ParseError{message: format!("expected \\n but got {}", x)})),
                    }
                }
                Some(x) => Err(Box::new(ParseError{message: format!("expected newline, comma, or space, but got {}", x)})),
            }
        }
    }
}

fn read(r: &mut impl Read) -> io::Result<Option<u8>> {
    let mut buf = [0;1];
    match r.read(&mut buf)? {
        1 => Ok(Some(buf[0])),
        _ => Ok(None),
    }
}

fn read_expected(r: &mut impl Read, expected: u8) -> Result<(), Box<dyn Error>> {
    match read(r)? {
        None => Err(Box::new(ParseError{message: format!("expected {} but got EOF", expected)})),
        Some(x) => if x == expected {
            Ok(())
        } else {
            Err(Box::new(ParseError{message: format!("expected {} but got {}", expected, x)}))
        },
    }
}

fn read_number(r: &mut impl Read, scale: u16) -> Result<u16, Box<dyn Error>> {
    match read(r)? {
        None => Err(Box::new(ParseError{message: "expected a digit but got EOF".to_string()})),
        Some(digit) => {
            let digit = parse_digit(digit)?;
            match scale {
                1 => Ok(digit),
                _ => Ok(digit * scale + read_number(r, scale / 10)?),
            }
        }
    }
}

fn read_year(r: &mut impl Read) -> Result<Option<u16>, Box<dyn Error>> {
    match read(r)? {
        None => Ok(None),
        Some(digit) => {
            let year = 1000 * parse_digit(digit)? + read_number(r, 100)?;
            Ok(Some(year))
        }
    }
}

fn parse_digit(digit: u8) -> Result<u16, Box<dyn Error>> {
    match digit {
        b'0' | b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' | b'9' => Ok((digit - b'0') as u16),
        x => Err(Box::new(ParseError{message: format!("expected a digit but got {}", x)})),
    }
}

#[cfg(test)]
mod tests {
    // variations:
    // - number of lines
    // - partial entry on last line vs complete entry
    // - time zone or not
    // - partial entry with/without comma

    use super::{Entry, Time, parse_entries};
    use std::io::{Cursor};

    #[test]
    fn test_empty() {
        let actual = parse_entries(Cursor::new(b"")).unwrap();
        assert_eq!(Vec::<Entry>::new(), actual);
    }

    #[test]
    fn test_start_no_tz() {
        let actual = parse_entries(Cursor::new(b"2020-01-02 12:34\n")).unwrap();
        assert_eq!(vec![
                   Entry{
                       start: Time{year: 2020, month: 1, day: 2, hour: 12, minute: 34, utc_offset: None},
                       stop: None,
                   },
        ], actual);
    }

    #[test]
    fn test_start_with_neg_tz() {
        let actual = parse_entries(Cursor::new(b"2020-01-02 12:34 -1001\n")).unwrap();
        assert_eq!(vec![
                   Entry{
                       start: Time{year: 2020, month: 1, day: 2, hour: 12, minute: 34, utc_offset: Some(-601)},
                       stop: None,
                   },
        ], actual);
    }

    #[test]
    fn test_start_with_pos_tz() {
        let actual = parse_entries(Cursor::new(b"2020-01-02 12:34 +1001\n")).unwrap();
        assert_eq!(vec![
                   Entry{
                       start: Time{year: 2020, month: 1, day: 2, hour: 12, minute: 34, utc_offset: Some(601)},
                       stop: None,
                   },
        ], actual);
    }

    #[test]
    fn test_single_entry() {
        let actual = parse_entries("2020-01-02 12:34 -0400,2020-01-02 13:34 -0400\n".as_bytes()).unwrap();
        assert_eq!(vec![
                   Entry{
                       start: Time{year: 2020, month: 1, day: 2, hour: 12, minute: 34, utc_offset: Some(240)},
                       stop: Some(Time{year: 2020, month: 1, day: 2, hour: 13, minute: 34, utc_offset: Some(-240)}),
                   },
        ], actual);
    }

    // test_single_entry_mixed_tz

    // test_several_entries

    // test_several_entries_last_is_partial
}
