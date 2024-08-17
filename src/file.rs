use crate::entry::{into_time_entries, Entry, TimeEntry};
use crate::parser::{parse_entries, parse_entry};
use crate::timesource::TimeSource;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;

const APPROX_LINE_LENGTH_FOR_SEEK: u64 = 50;

#[cfg(test)]
mod tests {
    use crate::entry::Entry;
    use crate::timesource::mock_time::mock_time;
    use crate::timesource::real_time::DefaultTimeSource;
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
        let ts = mock_time(date!(2020 - 07 - 15), time!(10:23), offset!(-04:00));
        let fixt = Fixture::new(None)?;
        assert_eq!(None, super::_start_new_entry(fixt.t_data_file(), &ts)?);
        assert_eq!("2020-07-15 10:23 -0400\n", fixt.read()?);
        Ok(())
    }

    #[test]
    fn test_start_in_empty_file() -> TestRes {
        let ts = mock_time(date!(2020 - 07 - 15), time!(10:23), offset!(-04:00));
        let fixt = Fixture::new(Some("empty.csv"))?;
        assert_eq!(None, super::_start_new_entry(fixt.t_data_file(), &ts)?);
        assert_eq!("2020-07-15 10:23 -0400\n", fixt.read()?);
        Ok(())
    }

    #[test]
    fn test_start_in_file_with_entries() -> TestRes {
        let ts = mock_time(date!(2020 - 08 - 08), time!(10:23), offset!(-04:00));
        let fixt = Fixture::new(Some("three-entries.csv"))?;
        assert_eq!(None, super::_start_new_entry(fixt.t_data_file(), &ts)?);
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
        let ts = mock_time(date!(2020 - 08 - 08), time!(10:23), offset!(-04:00));
        let fixt = Fixture::new(None)?;
        assert_eq!(None, super::_start_new_entry(fixt.t_data_file(), &ts)?);
        assert_eq!(Some(0), super::_start_new_entry(fixt.t_data_file(), &ts)?);
        let ts = mock_time(date!(2020 - 08 - 08), time!(10:34), offset!(-04:00));
        assert_eq!(Some(11), super::_start_new_entry(fixt.t_data_file(), &ts)?);
        assert_eq!("2020-08-08 10:23 -0400\n", fixt.read()?);
        Ok(())
    }

    #[test]
    fn test_start_with_blank_lines() -> TestRes {
        let ts = mock_time(date!(2020 - 08 - 08), time!(10:23), offset!(-04:00));
        let fixt = Fixture::new(Some("blank-lines.csv"))?;
        assert_eq!(None, super::_start_new_entry(fixt.t_data_file(), &ts)?);
        assert_eq!("2020-08-08 10:23 -0400\n", fixt.read()?);
        Ok(())
    }

    #[test]
    fn test_start_with_entry_and_trailing_whitespace() -> TestRes {
        let ts = mock_time(date!(2020 - 08 - 08), time!(10:23), offset!(-04:00));
        let fixt = Fixture::new(Some("entry-with-trailing-blank-lines.csv"))?;
        assert_eq!(None, super::_start_new_entry(fixt.t_data_file(), &ts)?);
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
        let ts = mock_time(date!(2020 - 08 - 08), time!(10:23), offset!(-04:00));
        let fixt = Fixture::new(Some("started-with-comma.csv"))?;
        assert_eq!(
            Some(1223),
            super::_start_new_entry(fixt.t_data_file(), &ts)?
        );
        Ok(())
    }

    #[test]
    fn test_start_when_entry_has_no_comma() -> TestRes {
        let ts = mock_time(date!(2020 - 08 - 08), time!(10:23), offset!(-04:00));
        let fixt = Fixture::new(Some("started-no-comma.csv"))?;
        assert_eq!(
            Some(1223),
            super::_start_new_entry(fixt.t_data_file(), &ts)?
        );
        Ok(())
    }

    #[test]
    fn test_start_when_two_entries_are_pending() -> TestRes {
        let ts = mock_time(date!(2020 - 08 - 07), time!(15:23), offset!(-04:00));
        let fixt = Fixture::new(Some("two-started-entries.csv"))?;
        assert_eq!(Some(83), super::_start_new_entry(fixt.t_data_file(), &ts)?);
        Ok(())
    }

    #[test]
    fn test_stop_in_new_file() -> TestRes {
        let fixt = Fixture::new(None)?;
        assert_eq!(
            None,
            super::_stop_current_entry(fixt.t_data_file(), &DefaultTimeSource)?
        );
        Ok(())
    }

    #[test]
    fn test_stop_in_empty_file() -> TestRes {
        let fixt = Fixture::new(Some("empty.csv"))?;
        assert_eq!(
            None,
            super::_stop_current_entry(fixt.t_data_file(), &DefaultTimeSource)?
        );
        Ok(())
    }

    #[test]
    fn test_stop_in_file_with_complete_entries() -> TestRes {
        let ts = mock_time(date!(2020 - 08 - 07), time!(23:23), offset!(-04:00));
        let fixt = Fixture::new(Some("three-entries.csv"))?;
        assert_eq!(
            Some((false, 328)),
            super::_stop_current_entry(fixt.t_data_file(), &ts)?
        );
        Ok(())
    }

    #[test]
    fn test_stop_when_entry_has_comma() -> TestRes {
        let ts = mock_time(date!(2020 - 08 - 07), time!(15:23), offset!(-04:00));
        let fixt = Fixture::new(Some("started-with-comma.csv"))?;
        assert_eq!(
            Some((true, 83)),
            super::_stop_current_entry(fixt.t_data_file(), &ts)?
        );
        assert_eq!(
            "2020-08-07 14:00 -0400,2020-08-07 15:23 -0400\n",
            fixt.read()?
        );
        Ok(())
    }

    #[test]
    fn test_stop_when_entry_has_no_comma() -> TestRes {
        let ts = mock_time(date!(2020 - 08 - 07), time!(15:23), offset!(-04:00));
        let fixt = Fixture::new(Some("started-no-comma.csv"))?;
        assert_eq!(
            Some((true, 83)),
            super::_stop_current_entry(fixt.t_data_file(), &ts)?
        );
        assert_eq!(
            "2020-08-07 14:00 -0400,2020-08-07 15:23 -0400\n",
            fixt.read()?
        );
        Ok(())
    }

    #[test]
    fn test_stop_when_entry_has_trailing_blank_lines() -> TestRes {
        let ts = mock_time(date!(2020 - 08 - 07), time!(15:23), offset!(-04:00));
        let fixt = Fixture::new(Some("started-trailing-blank-lines.csv"))?;
        assert_eq!(
            Some((true, 83)),
            super::_stop_current_entry(fixt.t_data_file(), &ts)?
        );
        assert_eq!("2020-08-07 14:00,2020-08-07 15:23 -0400\n", fixt.read()?);
        Ok(())
    }

    #[test]
    fn test_stop_when_two_entries_are_pending() -> TestRes {
        let ts = mock_time(date!(2020 - 08 - 07), time!(15:23), offset!(-04:00));
        let fixt = Fixture::new(Some("two-started-entries.csv"))?;
        assert_eq!(
            Some((true, 83)),
            super::_stop_current_entry(fixt.t_data_file(), &ts)?
        );
        Ok(())
    }

    #[test]
    fn test_read_last_entry_no_file() -> TestRes {
        let fixt = Fixture::new(None)?;
        assert_eq!(None, fixt.open()?.read_last_entry(&DefaultTimeSource)?);
        Ok(())
    }

    #[test]
    fn test_read_last_entry_empty_file() -> TestRes {
        let fixt = Fixture::new(Some("empty.csv"))?;
        assert_eq!(None, fixt.open()?.read_last_entry(&DefaultTimeSource)?);
        Ok(())
    }

    #[test]
    fn test_read_last_entry() -> TestRes {
        let fixt = Fixture::new(Some("three-entries.csv"))?;
        match fixt.open()?.read_last_entry(&DefaultTimeSource)? {
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
        assert_eq!(
            empty_entries(),
            fixt.open()?.read_last_entries(10, &DefaultTimeSource)?
        );
        Ok(())
    }

    #[test]
    fn test_read_last_entries_empty_file() -> TestRes {
        let fixt = Fixture::new(Some("empty.csv"))?;
        assert_eq!(
            empty_entries(),
            fixt.open()?.read_last_entries(10, &DefaultTimeSource)?
        );
        Ok(())
    }

    #[test]
    fn test_read_last_entries() -> TestRes {
        let fixt = Fixture::new(Some("thousand-entries.csv"))?;
        assert!(
            fixt.open()?
                .read_last_entries(10, &DefaultTimeSource)?
                .len()
                >= 10,
            "expect at least 10 entries to be returned"
        );
        assert!(
            fixt.open()?
                .read_last_entries(100, &DefaultTimeSource)?
                .len()
                >= 100,
            "expect at least 100 entries to be returned"
        );
        // the file only has 1000 entries, so we can't get more than that.
        assert_eq!(
            1000,
            fixt.open()?
                .read_last_entries(1000, &DefaultTimeSource)?
                .len()
        );
        assert_eq!(
            1000,
            fixt.open()?
                .read_last_entries(10000, &DefaultTimeSource)?
                .len()
        );
        Ok(())
    }

    #[test]
    fn test_read_entries_no_file() -> TestRes {
        let fixt = Fixture::new(None)?;
        assert_eq!(
            empty_entries(),
            fixt.open()?.read_entries(&DefaultTimeSource)?
        );
        Ok(())
    }

    #[test]
    fn test_read_entries_empty_file() -> TestRes {
        let fixt = Fixture::new(Some("empty.csv"))?;
        assert_eq!(
            empty_entries(),
            fixt.open()?.read_entries(&DefaultTimeSource)?
        );
        Ok(())
    }

    #[test]
    fn test_read_entries_blank_file() -> TestRes {
        let fixt = Fixture::new(Some("blank-lines.csv"))?;
        assert_eq!(
            empty_entries(),
            fixt.open()?.read_entries(&DefaultTimeSource)?
        );
        Ok(())
    }

    #[test]
    fn test_read_entries_three() -> TestRes {
        let fixt = Fixture::new(Some("three-entries.csv"))?;
        assert_eq!(3, fixt.open()?.read_entries(&DefaultTimeSource)?.len());
        Ok(())
    }

    #[test]
    fn test_read_entries_thousand() -> TestRes {
        let fixt = Fixture::new(Some("thousand-entries.csv"))?;
        assert_eq!(1000, fixt.open()?.read_entries(&DefaultTimeSource)?.len());
        Ok(())
    }

    #[test]
    fn test_read_entries_started() -> TestRes {
        let fixt = Fixture::new(Some("started-with-comma.csv"))?;
        let entries = fixt.open()?.read_entries(&DefaultTimeSource)?;
        assert_eq!(1, entries.len());
        assert!(!entries[0].time().is_finished());
        Ok(())
    }
}

pub fn t_data_file() -> Result<String, std::env::VarError> {
    std::env::var("T_DATA_FILE")
}

// If there isn't a pending entry, start a new one.
pub fn start_new_entry<TS: TimeSource>(ts: &TS) -> Result<Option<i64>, Box<dyn Error>> {
    _start_new_entry(t_data_file()?, ts)
}

fn _start_new_entry<P: AsRef<Path>, TS: TimeSource>(
    t_data_file: P,
    ts: &TS,
) -> Result<Option<i64>, Box<dyn Error>> {
    let (mut f, entry, _, pos) = read_for_update(t_data_file, ts)?;
    if let Some(entry) = entry {
        if !entry.is_finished() {
            return Ok(Some(entry.minutes(ts)));
        }
    }
    f.seek(SeekFrom::Start(pos))?;
    write!(f, "{}", TimeEntry::start(ts))?;
    Ok(None)
}

// If there is a pending entry, finish it.
pub fn stop_current_entry<TS: TimeSource>(ts: &TS) -> Result<Option<(bool, i64)>, Box<dyn Error>> {
    _stop_current_entry(t_data_file()?, ts)
}

fn _stop_current_entry<P: AsRef<Path>, TS: TimeSource>(
    t_data_file: P,
    ts: &TS,
) -> Result<Option<(bool, i64)>, Box<dyn Error>> {
    let (mut f, entry, pos, _) = read_for_update(t_data_file, ts)?;
    match entry {
        None => Ok(None),
        Some(entry) => {
            if entry.is_finished() {
                Ok(Some((false, entry.minutes_since_stop(ts).unwrap())))
            } else {
                f.seek(SeekFrom::Start(pos))?;
                let entry = entry.finish(ts);
                write!(f, "{}", entry)?;
                Ok(Some((true, entry.minutes(ts))))
            }
        }
    }
}

type ReadResult = (File, Option<TimeEntry>, u64, u64);

// Get the last entry from the file, along with its start and stop
// positions.
fn read_for_update<P: AsRef<Path>, TS: TimeSource>(
    t_data_file: P,
    ts: &TS,
) -> Result<ReadResult, Box<dyn Error>> {
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(t_data_file)?;

    seek_last_entries(&mut f, 2)?;

    let mut r = BufReader::new(f);
    let mut last_entry = None;
    let mut last_time_entry_is_finished = true;
    let mut start_pos = get_pos(&mut r)?;
    let mut stop_pos = start_pos;

    loop {
        let (entry, returned) = parse_entry(r, ts)?;
        r = returned;
        match entry {
            None => break,
            Some(Entry::Time(te)) => {
                last_time_entry_is_finished = te.is_finished();
                last_entry = Some(te);
            }
            _ => {
                last_entry = None;
            }
        };
        start_pos = stop_pos;
        stop_pos = get_pos(&mut r)?;
    }

    if last_entry.is_none() && !last_time_entry_is_finished {
        panic!("last time entry is open and annotation comes after it (todo - make this an Err)");
    }

    Ok((r.into_inner(), last_entry, start_pos, stop_pos))
}

pub fn read_entries<TS: TimeSource>(ts: &TS) -> Result<Vec<Entry>, Box<dyn Error>> {
    t_open(t_data_file()?)?.read_entries(ts)
}

pub fn read_time_entries<TS: TimeSource>(ts: &TS) -> Result<Vec<TimeEntry>, Box<dyn Error>> {
    read_entries(ts).map(into_time_entries)
}

pub fn read_last_entry<TS: TimeSource>(ts: &TS) -> Result<Option<Entry>, Box<dyn Error>> {
    t_open(t_data_file()?)?.read_last_entry(ts)
}

pub fn read_last_entries<TS: TimeSource>(n: u64, ts: &TS) -> Result<Vec<Entry>, Box<dyn Error>> {
    t_open(t_data_file()?)?.read_last_entries(n, ts)
}

pub fn t_open<P: AsRef<Path>>(t_data_file: P) -> io::Result<TFile> {
    match File::open(t_data_file) {
        Ok(f) => Ok(TFile { f: Some(f) }),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Ok(TFile { f: None }),
            _ => Err(e),
        },
    }
}

pub struct TFile {
    f: Option<File>,
}

impl TFile {
    pub fn read_entries<TS: TimeSource>(self, ts: &TS) -> Result<Vec<Entry>, Box<dyn Error>> {
        match self.f {
            Some(f) => parse_entries(f, ts),
            None => Ok(vec![]),
        }
    }

    fn read_last_entry<TS: TimeSource>(self, ts: &TS) -> Result<Option<Entry>, Box<dyn Error>> {
        Ok(self.read_last_entries(1, ts)?.into_iter().last())
    }

    pub fn read_last_entries<TS: TimeSource>(
        self,
        n: u64,
        ts: &TS,
    ) -> Result<Vec<Entry>, Box<dyn Error>> {
        match self.f {
            None => Ok(vec![]),
            Some(mut f) => {
                seek_last_entries(&mut f, n)?;
                parse_entries(f, ts)
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

fn get_pos<S: Seek>(mut f: S) -> io::Result<u64> {
    f.stream_position()
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
