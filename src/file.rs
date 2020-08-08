use crate::entry::Entry;
use crate::parser::{parse_entries, parse_entry};
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{self, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;

const APPROX_LINE_LENGTH_FOR_SEEK: u64 = 50;

#[cfg(test)]
mod tests {
    use crate::entry::mock_time::*;
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

        fn read(&self) -> std::io::Result<String> {
            let mut f = File::open(self.t_data_file())?;
            let mut res = String::new();
            f.read_to_string(&mut res)?;
            Ok(res)
        }
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
        let fixt = Fixture::new(Some("empty.csv"))?;
        assert_eq!(None, super::_start_new_entry(fixt.t_data_file())?);
        let entries = super::_read_entries(fixt.t_data_file())?;
        assert_eq!(1, entries.len(), "should have one entry: {:?}", entries);
        assert!(
            !entries[0].is_finished(),
            "should not be finished: {}",
            entries[0]
        );
        Ok(())
    }

    #[test]
    fn test_start_in_file_with_entries() -> TestRes {
        let fixt = Fixture::new(Some("three-entries.csv"))?;
        assert_eq!(None, super::_start_new_entry(fixt.t_data_file())?);
        let entries = super::_read_entries(fixt.t_data_file())?;
        assert_eq!(4, entries.len(), "should have four entries: {:?}", entries);
        assert!(
            entries[0].is_finished(),
            "[0] should be finished: {}",
            entries[0]
        );
        assert!(
            entries[1].is_finished(),
            "[1] should be finished: {}",
            entries[1]
        );
        assert!(
            entries[2].is_finished(),
            "[2] should be finished: {}",
            entries[2]
        );
        assert!(
            !entries[3].is_finished(),
            "[3] (new!) should not be finished: {}",
            entries[3]
        );
        Ok(())
    }

    #[test]
    fn test_start_twice() -> TestRes {
        let fixt = Fixture::new(None)?;
        assert_eq!(None, super::_start_new_entry(fixt.t_data_file())?);
        assert_eq!(Some(0), super::_start_new_entry(fixt.t_data_file())?);
        let entries = super::_read_entries(fixt.t_data_file())?;
        assert_eq!(1, entries.len(), "should have one entry: {:?}", entries);
        assert!(
            !entries[0].is_finished(),
            "should not be finished: {}",
            entries[0]
        );
        Ok(())
    }
}

pub fn t_data_file() -> String {
    std::env::var("T_DATA_FILE").unwrap()
}

pub fn start_new_entry() -> Result<Option<i64>, Box<dyn Error>> {
    _start_new_entry(t_data_file())
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
    _read_entries(t_data_file())
}

fn _read_entries<P: AsRef<Path>>(t_data_file: P) -> Result<Vec<Entry>, Box<dyn Error>> {
    t_open(t_data_file)?.read_entries()
}

pub fn read_last_entry() -> Result<Option<Entry>, Box<dyn Error>> {
    t_open(t_data_file())?.read_last_entry()
}

pub fn read_last_entries(n: u64) -> Result<Vec<Entry>, Box<dyn Error>> {
    t_open(t_data_file())?.read_last_entries(n)
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
