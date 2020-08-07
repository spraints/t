use crate::entry::Entry;
use crate::parser::{parse_entries, parse_entry};
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{self, ErrorKind, Read, Seek, SeekFrom, Write};

const APPROX_LINE_LENGTH_FOR_SEEK: u64 = 50;

pub fn t_data_file() -> String {
    std::env::var("T_DATA_FILE").unwrap()
}

pub fn start_new_entry() -> Result<Option<i64>, Box<dyn Error>> {
    let f = OpenOptions::new()
        .read(true)
        .write(true)
        .open(t_data_file());
    let mut f = match f {
        Ok(mut f) => {
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
            f
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => File::create(t_data_file())?,
            _ => return Err(Box::new(e)),
        },
    };
    write!(f, "{}", Entry::start())?;
    Ok(None)
}

pub fn read_entries() -> Result<Vec<Entry>, Box<dyn Error>> {
    t_open()?.read_entries()
}

pub fn read_last_entry() -> Result<Option<Entry>, Box<dyn Error>> {
    t_open()?.read_last_entry()
}

pub fn read_last_entries(n: u64) -> Result<Vec<Entry>, Box<dyn Error>> {
    t_open()?.read_last_entries(n)
}

fn t_open() -> io::Result<TFile> {
    match File::open(t_data_file()) {
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
