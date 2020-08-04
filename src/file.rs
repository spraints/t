use crate::entry::Entry;
use crate::parser::parse_entries;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

const APPROX_LINE_LENGTH_FOR_SEEK: u64 = 50;

pub fn t_data_file() -> String {
    std::env::var("T_DATA_FILE").unwrap()
}

pub fn read_entries() -> Result<Vec<Entry>, Box<dyn Error>> {
    _read_entries(|f| Ok(f))
}

pub fn read_last_entry() -> Result<Option<Entry>, Box<dyn Error>> {
    Ok(read_last_entries(1)?.into_iter().last())
}

pub fn read_last_entries(n: u64) -> Result<Vec<Entry>, Box<dyn Error>> {
    _read_entries(|mut f| {
        let len = f.metadata()?.len();
        let off = n * APPROX_LINE_LENGTH_FOR_SEEK;
        if off < len {
            f.seek(SeekFrom::Start(len - off))?;
            read_to(&mut f, b'\n')?;
        }
        Ok(f)
    })
}

fn _read_entries<F>(seek: F) -> Result<Vec<Entry>, Box<dyn Error>>
where
    F: Fn(File) -> Result<File, Box<dyn Error>>,
{
    match File::open(t_data_file()) {
        Err(_) => Ok(vec![]),
        Ok(f) => parse_entries(seek(f)?),
    }
}

fn read_to(f: &mut File, c: u8) -> Result<(), Box<dyn Error>> {
    let mut buf = [0; 1];
    loop {
        let res = f.read(&mut buf)?;
        if res == 0 || buf[0] == c {
            return Ok(());
        }
    }
}
