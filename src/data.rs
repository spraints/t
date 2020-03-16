use chrono::offset::{Local, TimeZone};
use chrono::DateTime;
use std::fmt::Write;
use std::io::BufRead;

const DATE_FORMAT: &str = "%Y-%m-%d";
const TIME_FORMAT: &str = "%Y-%m-%d %H:%M %z";
const OLD_TIME_FORMAT: &str = "%Y-%m-%d %H:%M";

//  DEFAULT_SPARKS = %w(▁ ▂ ▃ ▄ ▅ ▆ ▇ )

pub struct Data {
    entries: Vec<Entry>,
}

#[derive(Debug, PartialEq)]
struct Entry {
    raw: String,
    start: DateTime<Local>,
    end: Option<DateTime<Local>>,
}

pub fn read<T: BufRead>(file: T) -> Result<Data, String> {
    let mut entries = vec![];
    for line in file.lines() {
        match line {
            Err(err) => return Err(format!("{}", err)),
            Ok(line) => match parse_entry(line) {
                Err(err) => return Err(err),
                Ok(None) => (),
                Ok(Some(entry)) => entries.push(entry),
            },
        }
    }
    Ok(Data { entries })
}

impl Data {
    pub fn write<W: Write>(&self, w: &mut W) -> Result<(), String> {
        for e in &self.entries {
            w.write_fmt(format_args!("{}\n", e.raw.trim()))
                .map_err(|err| format!("{}", err))?;
        }
        Ok(())
    }
}

fn parse_entry(line: String) -> Result<Option<Entry>, String> {
    let parts: Vec<&str> = line.split(",").map(|s| s.trim()).collect();
    match parts.len() {
        0 => return Ok(None),
        1 => Ok(Some(Entry {
            raw: line.trim().to_string(),
            start: parse_time(parts[0])?,
            end: None,
        })),
        2 => Ok(Some(Entry {
            raw: line.trim().to_string(),
            start: parse_time(parts[0])?,
            end: maybe_parse_time(parts[1])?,
        })),
        _ => Err(format!("unrecognized entry line: {}", line)),
    }
}

fn parse_time(s: &str) -> Result<DateTime<Local>, String> {
    let res = real_parse_time(s);
    if let Ok(dt) = res {
        println!("{} -> {} ({})", s, dt, dt.timestamp());
    }
    res
}

fn real_parse_time(s: &str) -> Result<DateTime<Local>, String> {
    match Local.datetime_from_str(s, OLD_TIME_FORMAT) {
        Ok(dt) => Ok(dt.with_timezone(&Local)),
        Err(e1) => match DateTime::parse_from_str(s, TIME_FORMAT) {
            Ok(dt) => Ok(dt.with_timezone(&Local)),
            Err(e2) => Err(format!("could not parse {}: {}, {}", s, e1, e2)),
        },
    }
}

fn maybe_parse_time(s: &str) -> Result<Option<DateTime<Local>>, String> {
    match s {
        "" => Ok(None),
        _ => Ok(Some(parse_time(s)?)),
    }
}

#[cfg(test)]
mod tests {
    use super::{Data, Entry};
    use chrono::offset::{Local, TimeZone};

    fn parse_data<'a>(s: &'a str) -> Result<Data, String> {
        let reader = &mut s.as_bytes();
        super::read(reader)
    }

    fn parse_entries<'a>(s: &'a str) -> Result<Vec<Entry>, String> {
        Ok(parse_data(s)?.entries)
    }

    #[test]
    fn test_parse_empty() -> Result<(), String> {
        assert_eq!(parse_entries("")?, vec![]);
        Ok(())
    }

    #[test]
    fn test_parse_partial_no_comma_no_tz() -> Result<(), String> {
        assert_eq!(
            parse_entries("2013-09-05 11:39")?,
            vec![Entry {
                raw: "2013-09-05 11:39".to_string(),
                start: Local.timestamp(1378381140, 0),
                end: None,
            },]
        );
        Ok(())
    }

    #[test]
    fn test_parse_partial_no_tz() -> Result<(), String> {
        assert_eq!(
            parse_entries("2013-09-05 11:39,")?,
            vec![Entry {
                raw: "2013-09-05 11:39,".to_string(),
                start: Local.timestamp(1378381140, 0),
                end: None,
            },]
        );
        Ok(())
    }

    #[test]
    fn test_parse_one_no_nl_no_tz() -> Result<(), String> {
        assert_eq!(
            parse_entries("2013-09-05 11:39,2013-09-05 11:49")?,
            vec![Entry {
                raw: "2013-09-05 11:39,2013-09-05 11:49".to_string(),
                start: Local.timestamp(1378381140, 0),
                end: Some(Local.timestamp(1378381740, 0)),
            },]
        );
        Ok(())
    }

    #[test]
    fn test_parse_one_no_tz() -> Result<(), String> {
        assert_eq!(
            parse_entries("2013-09-05 11:39,2013-09-05 11:49\n")?,
            vec![Entry {
                raw: "2013-09-05 11:39,2013-09-05 11:49".to_string(),
                start: Local.timestamp(1378381140, 0),
                end: Some(Local.timestamp(1378381740, 0)),
            },]
        );
        Ok(())
    }

    #[test]
    fn test_parse_one_mixed_tz() -> Result<(), String> {
        assert_eq!(
            parse_entries("2013-09-05 11:39 -0000,2013-09-05 11:49 -1000\n")?,
            vec![Entry {
                raw: "2013-09-05 11:39 -0000,2013-09-05 11:49 -1000".to_string(),
                start: Local.timestamp(1378381140, 0),
                end: Some(Local.timestamp(1378417740, 0)),
            },]
        );
        Ok(())
    }

    #[test]
    fn test_parse() -> Result<(), String> {
        assert_eq!(
            parse_entries(
                "2013-09-05 11:39 -0000,2013-09-05 11:49 -1000\n\
                 2013-09-07 10:10, 2013-09-07 11:11\n"
            )?,
            vec![
                Entry {
                    raw: "2013-09-05 11:39 -0000,2013-09-05 11:49 -1000".to_string(),
                    start: Local.timestamp(1378381140, 0),
                    end: Some(Local.timestamp(1378417740, 0)),
                },
                Entry {
                    raw: "2013-09-07 10:10, 2013-09-07 11:11".to_string(),
                    start: Local.timestamp(1378548600, 0),
                    end: Some(Local.timestamp(1378552260, 0)),
                },
            ]
        );
        Ok(())
    }

    #[test]
    fn test_roundtrip() -> Result<(), String> {
        let input = "2013-09-05 11:39 -0000,2013-09-05 11:49 -1000\n\
                     2013-09-07 10:10, 2013-09-07 11:11\n";
        let mut output = String::new();
        parse_data(input)?.write(&mut output)?;
        assert_eq!(input, output);
        Ok(())
    }
}
