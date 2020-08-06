use crate::entry::Entry;
use crate::parser::parse_entries;
use std::error::Error;
use std::fs::File;
use std::io::{self, ErrorKind, Read, Seek, SeekFrom};

const APPROX_LINE_LENGTH_FOR_SEEK: u64 = 50;

pub fn t_data_file() -> String {
    std::env::var("T_DATA_FILE").unwrap()
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
                let len = f.metadata()?.len();
                let off = n * APPROX_LINE_LENGTH_FOR_SEEK;
                if off < len {
                    f.seek(SeekFrom::Start(len - off))?;
                    read_to(&mut f, b'\n')?;
                }
                parse_entries(f)
            }
        }
    }
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
