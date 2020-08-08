use crate::entry::Entry;
use crate::parser::{parse_entries, parse_entry};
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{self, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;

const APPROX_LINE_LENGTH_FOR_SEEK: u64 = 50;

#[cfg(test)]
mod tests {
    use crate::entry::{mock_time::*, Entry};
    use std::error::Error;
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;
    use time::{date, offset, time};

    struct Fixture {
        dir: tempfile::TempDir,
    }

    impl Fixture {
        fn new(name: Option<&str>) -> Result<Self, Box<dyn Error>> {
            let ret = Self {
                dir: tempfile::tempdir()?,
            };
            if let Some(name) = name {
                let root_dir = &std::env::var("CARGO_MANIFEST_DIR")?;
                let mut source_path = PathBuf::from(root_dir);
                source_path.push("tests/fixtures/file");
                source_path.push(name);

                std::fs::copy(source_path, ret.t_data_file())?;
            }
            Ok(ret)
        }

        fn t_data_file(&self) -> PathBuf {
            self.dir.path().join("test-t.csv")
        }

        fn open(&self) -> std::io::Result<super::TFile> {
            super::t_open(self.t_data_file())
        }

        fn read(&self) -> std::io::Result<String> {
            let mut f = File::open(self.t_data_file())?;
            let mut res = String::new();
            f.read_to_string(&mut res)?;
            Ok(res)
        }
    }

    fn empty_entries() -> Vec<Entry> {
        vec![]
    }

    type TestRes = Result<(), Box<dyn Error>>;

    #[test]
    fn test_start_in_new_file() -> TestRes {
        set_mock_time(date!(2020 - 07 - 15), time!(10:23), offset!(-04:00));
        let fixt = Fixture::new(None)?;
        assert_eq!(None, super::_start_new_entry(fixt.t_data_file())?);
        assert_eq!("2020-07-15 10:23 -0400\n", fixt.read()?);
        Ok(())
    }

    #[test]
    fn test_start_in_empty_file() -> TestRes {
        set_mock_time(date!(2020 - 07 - 15), time!(10:23), offset!(-04:00));
        let fixt = Fixture::new(Some("empty.csv"))?;
        assert_eq!(None, super::_start_new_entry(fixt.t_data_file())?);
        assert_eq!("2020-07-15 10:23 -0400\n", fixt.read()?);
        Ok(())
    }

    #[test]
    fn test_start_in_file_with_entries() -> TestRes {
        set_mock_time(date!(2020 - 08 - 08), time!(10:23), offset!(-04:00));
        let fixt = Fixture::new(Some("three-entries.csv"))?;
        assert_eq!(None, super::_start_new_entry(fixt.t_data_file())?);
        assert_eq!(
            "2020-08-06 15:38 -0400,2020-08-06 18:40 -0400\n\
                    2020-08-07 11:03 -0400,2020-08-07 13:07 -0400\n\
                    2020-08-07 14:00 -0400,2020-08-07 17:55 -0400\n\
                    2020-08-08 10:23 -0400\n",
            fixt.read()?
        );
        Ok(())
    }

    #[test]
    fn test_start_twice() -> TestRes {
        set_mock_time(date!(2020 - 08 - 08), time!(10:23), offset!(-04:00));
        let fixt = Fixture::new(None)?;
        assert_eq!(None, super::_start_new_entry(fixt.t_data_file())?);
        assert_eq!(Some(0), super::_start_new_entry(fixt.t_data_file())?);
        set_mock_time(date!(2020 - 08 - 08), time!(10:34), offset!(-04:00));
        assert_eq!(Some(11), super::_start_new_entry(fixt.t_data_file())?);
        assert_eq!("2020-08-08 10:23 -0400\n", fixt.read()?);
        Ok(())
    }

    #[test]
    fn test_start_with_blank_lines() -> TestRes {
        set_mock_time(date!(2020 - 08 - 08), time!(10:23), offset!(-04:00));
        let fixt = Fixture::new(Some("blank-lines.csv"))?;
        assert_eq!(None, super::_start_new_entry(fixt.t_data_file())?);
        assert_eq!("2020-08-08 10:23 -0400\n", fixt.read()?);
        Ok(())
    }

    #[test]
    fn test_start_with_entry_and_trailing_whitespace() -> TestRes {
        set_mock_time(date!(2020 - 08 - 08), time!(10:23), offset!(-04:00));
        let fixt = Fixture::new(Some("entry-with-trailing-blank-lines.csv"))?;
        assert_eq!(None, super::_start_new_entry(fixt.t_data_file())?);
        assert_eq!(
            "2020-08-06 14:00 -0400,2020-08-06 17:55 -0400\n\
                    \n\
                    2020-08-07 14:00 -0400,2020-08-07 17:55 -0400\n\
                    2020-08-08 10:23 -0400\n",
            fixt.read()?
        );
        Ok(())
    }

    #[test]
    fn test_start_when_entry_has_comma() -> TestRes {
        set_mock_time(date!(2020 - 08 - 08), time!(10:23), offset!(-04:00));
        let fixt = Fixture::new(Some("started-with-comma.csv"))?;
        assert_eq!(Some(1223), super::_start_new_entry(fixt.t_data_file())?);
        Ok(())
    }

    #[test]
    fn test_start_when_entry_has_no_comma() -> TestRes {
        set_mock_time(date!(2020 - 08 - 08), time!(10:23), offset!(-04:00));
        let fixt = Fixture::new(Some("started-no-comma.csv"))?;
        assert_eq!(Some(1223), super::_start_new_entry(fixt.t_data_file())?);
        Ok(())
    }

    #[test]
    fn test_read_last_entry_no_file() -> TestRes {
        let fixt = Fixture::new(None)?;
        assert_eq!(None, fixt.open()?.read_last_entry()?);
        Ok(())
    }

    #[test]
    fn test_read_last_entry_empty_file() -> TestRes {
        let fixt = Fixture::new(Some("empty.csv"))?;
        assert_eq!(None, fixt.open()?.read_last_entry()?);
        Ok(())
    }

    #[test]
    fn test_read_last_entry() -> TestRes {
        let fixt = Fixture::new(Some("three-entries.csv"))?;
        match fixt.open()?.read_last_entry()? {
            None => panic!("expected an entry"),
            Some(e) => assert_eq!(
                "2020-08-07 14:00 -0400,2020-08-07 17:55 -0400\n",
                format!("{}", e)
            ),
        };
        Ok(())
    }

    #[test]
    fn test_read_last_entries_no_file() -> TestRes {
        let fixt = Fixture::new(None)?;
        assert_eq!(empty_entries(), fixt.open()?.read_last_entries(10)?);
        Ok(())
    }

    #[test]
    fn test_read_last_entries_empty_file() -> TestRes {
        let fixt = Fixture::new(Some("empty.csv"))?;
        assert_eq!(empty_entries(), fixt.open()?.read_last_entries(10)?);
        Ok(())
    }

    #[test]
    fn test_read_last_entries() -> TestRes {
        let fixt = Fixture::new(Some("thousand-entries.csv"))?;
        assert!(
            fixt.open()?.read_last_entries(10)?.len() >= 10,
            "expect at least 10 entries to be returned"
        );
        assert!(
            fixt.open()?.read_last_entries(100)?.len() >= 100,
            "expect at least 100 entries to be returned"
        );
        // the file only has 1000 entries, so we can't get more than that.
        assert_eq!(1000, fixt.open()?.read_last_entries(1000)?.len());
        assert_eq!(1000, fixt.open()?.read_last_entries(10000)?.len());
        Ok(())
    }
}

pub fn t_data_file() -> Result<String, std::env::VarError> {
    std::env::var("T_DATA_FILE")
}

pub fn start_new_entry() -> Result<Option<i64>, Box<dyn Error>> {
    _start_new_entry(t_data_file()?)
}

fn _start_new_entry<P: AsRef<Path>>(t_data_file: P) -> Result<Option<i64>, Box<dyn Error>> {
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(t_data_file)?;
    seek_last_entries(&mut f, 1)?;
    let mut pos = get_pos(&mut f)?;
    loop {
        let (entry, returned) = parse_entry(f)?;
        f = returned;
        match entry {
            None => break,
            Some(entry) => {
                if !entry.is_finished() {
                    return Ok(Some(entry.minutes()));
                }
            }
        };
        pos = get_pos(&mut f)?;
    }
    f.seek(SeekFrom::Start(pos))?;
    write!(f, "{}", Entry::start())?;
    Ok(None)
}

pub fn read_entries() -> Result<Vec<Entry>, Box<dyn Error>> {
    t_open(t_data_file()?)?.read_entries()
}

pub fn read_last_entry() -> Result<Option<Entry>, Box<dyn Error>> {
    t_open(t_data_file()?)?.read_last_entry()
}

pub fn read_last_entries(n: u64) -> Result<Vec<Entry>, Box<dyn Error>> {
    t_open(t_data_file()?)?.read_last_entries(n)
}

fn t_open<P: AsRef<Path>>(t_data_file: P) -> io::Result<TFile> {
    match File::open(t_data_file) {
        Ok(f) => Ok(TFile { f: Some(f) }),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Ok(TFile { f: None }),
            _ => Err(e),
        },
    }
}

struct TFile {
    f: Option<File>,
}

impl TFile {
    fn read_entries(self) -> Result<Vec<Entry>, Box<dyn Error>> {
        match self.f {
            Some(f) => parse_entries(f),
            None => Ok(vec![]),
        }
    }

    fn read_last_entry(self) -> Result<Option<Entry>, Box<dyn Error>> {
        Ok(self.read_last_entries(1)?.into_iter().last())
    }

    fn read_last_entries(self, n: u64) -> Result<Vec<Entry>, Box<dyn Error>> {
        match self.f {
            None => Ok(vec![]),
            Some(mut f) => {
                seek_last_entries(&mut f, n)?;
                parse_entries(f)
            }
        }
    }
}

fn seek_last_entries(f: &mut File, n: u64) -> io::Result<()> {
    let len = f.metadata()?.len();
    let off = n * APPROX_LINE_LENGTH_FOR_SEEK;
    if off < len {
        f.seek(SeekFrom::Start(len - off))?;
        read_to(f, b'\n')?;
    }
    Ok(())
}

fn get_pos(f: &mut File) -> io::Result<u64> {
    f.seek(SeekFrom::Current(0))
}

fn read_to(f: &mut File, c: u8) -> io::Result<()> {
    let mut buf = [0; 1];
    loop {
        let res = f.read(&mut buf)?;
        if res == 0 || buf[0] == c {
            return Ok(());
        }
    }
}
