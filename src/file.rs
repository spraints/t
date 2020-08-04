use crate::entry::Entry;
use crate::parser::parse_entries;
use std::error::Error;
use std::fs::File;

pub fn t_data_file() -> String {
    std::env::var("T_DATA_FILE").unwrap()
}

pub fn read_entries() -> Result<Vec<Entry>, Box<dyn Error>> {
    match File::open(t_data_file()) {
        Err(_) => Ok(vec![]),
        Ok(f) => parse_entries(f),
    }
}

pub fn read_last_entry() -> Result<Option<Entry>, Box<dyn Error>> {
    // TODO - seek close to the end
    Ok(read_entries()?.into_iter().last())
}

pub fn read_last_entries(_: usize) -> Result<Vec<Entry>, Box<dyn Error>> {
    // TODO
    read_entries()
}
