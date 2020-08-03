use crate::entry::Entry;
use crate::parser::parse_entries;
use std::error::Error;
use std::fs::File;

pub fn t_data_file() -> String {
    std::env::var("T_DATA_FILE").unwrap()
}

pub fn read_entries() -> Result<Vec<Entry>, Box<dyn Error>> {
    let f = File::open(t_data_file())?;
    parse_entries(f)
}

pub fn read_last_entry() -> Result<Option<Entry>, Box<dyn Error>> {
    Ok(read_entries()?.into_iter().last())
}
