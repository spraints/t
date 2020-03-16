use chrono::offset::{Local, TimeZone};
use chrono::DateTime;
use std::fmt::{Display, Write};
use std::io::BufRead;

//const DATE_FORMAT: &str = "%Y-%m-%d";
const TIME_FORMAT: &str = "%Y-%m-%d %H:%M %z";
const OLD_TIME_FORMAT: &str = "%Y-%m-%d %H:%M";

//  DEFAULT_SPARKS = %w(▁ ▂ ▃ ▄ ▅ ▆ ▇ )

pub struct Data {
    entries: Vec<Entry>,
}

#[derive(Debug, PartialEq)]
struct Entry {
    raw: Option<String>,
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
            e.write(w)?;
        }

        Ok(())
    }

    pub fn start(&mut self, start: DateTime<Local>) {
        self.entries.push(Entry {
            raw: None,
            end: None,
            start,
        });
    }
}

impl Entry {
    fn write<W: Write>(&self, w: &mut W) -> Result<(), String> {
        fn converr<T, E: Display>(res: Result<T, E>) -> Result<T, String> {
            res.map_err(|err| format!("{}", err))
        }

        match &self.raw {
            Some(s) => converr(w.write_fmt(format_args!("{}\n", s)))?,
            None => {
                converr(w.write_fmt(format_args!("{},", self.start.format(TIME_FORMAT))))?;
                if let Some(end) = &self.end {
                    converr(w.write_fmt(format_args!("{}", end.format(TIME_FORMAT))))?;
                }
                converr(w.write_str("\n"))?;
            }
        }

        Ok(())
    }

    pub fn stop(&mut self, dt: DateTime<Local>) {
        self.end = Some(dt);
    }
}

fn parse_entry(line: String) -> Result<Option<Entry>, String> {
    let parts: Vec<&str> = line.split(",").map(|s| s.trim()).collect();
    match parts.len() {
        0 => return Ok(None),
        1 => Ok(Some(Entry {
            raw: Some(line.trim().to_string()),
            start: parse_time(parts[0])?,
            end: None,
        })),
        2 => Ok(Some(Entry {
            raw: Some(line.trim().to_string()),
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
    // Note: many of these tests only work if TZ is UTC.

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
                raw: Some("2013-09-05 11:39".to_string()),
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
                raw: Some("2013-09-05 11:39,".to_string()),
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
                raw: Some("2013-09-05 11:39,2013-09-05 11:49".to_string()),
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
                raw: Some("2013-09-05 11:39,2013-09-05 11:49".to_string()),
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
                raw: Some("2013-09-05 11:39 -0000,2013-09-05 11:49 -1000".to_string()),
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
                    raw: Some("2013-09-05 11:39 -0000,2013-09-05 11:49 -1000".to_string()),
                    start: Local.timestamp(1378381140, 0),
                    end: Some(Local.timestamp(1378417740, 0)),
                },
                Entry {
                    raw: Some("2013-09-07 10:10, 2013-09-07 11:11".to_string()),
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

    #[test]
    fn test_format() -> Result<(), String> {
        let data = Data {
            entries: vec![
                Entry {
                    raw: None,
                    start: Local.timestamp(1378381140, 0),
                    end: Some(Local.timestamp(1378417740, 0)),
                },
                Entry {
                    raw: None,
                    start: Local.timestamp(1378548600, 0),
                    end: None,
                },
            ],
        };
        let mut output = String::new();
        data.write(&mut output)?;
        assert_eq!(
            output,
            "2013-09-05 11:39 +0000,2013-09-05 21:49 +0000\n\
             2013-09-07 10:10 +0000,\n"
        );
        Ok(())
    }

    #[test]
    fn test_start() {
        let mut data = Data { entries: vec![] };
        data.start(Local.timestamp(1371371371, 0));
        assert_eq!(
            data.entries,
            vec![Entry {
                raw: None,
                start: Local.timestamp(1371371371, 0),
                end: None,
            }]
        );
    }

    #[test]
    fn test_stop() {
        let mut entry = Entry {
            raw: None,
            start: Local.timestamp(1231231231, 0),
            end: None,
        };
        entry.stop(Local.timestamp(1234123412, 0));
        assert_eq!(
            entry,
            Entry {
                raw: None,
                start: Local.timestamp(1231231231, 0),
                end: Some(Local.timestamp(1234123412, 0)),
            }
        );
    }
}
