#[derive(Debug, PartialEq)]
pub struct Entry {
    pub start: Time,
    pub stop: Option<Time>,
}

#[derive(Debug, PartialEq)]
pub struct Time {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub utc_offset: Option<i16>,
}
