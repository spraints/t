use time::{time, OffsetDateTime, PrimitiveDateTime};

pub fn today() -> (OffsetDateTime, OffsetDateTime) {
    let now = OffsetDateTime::now_local();
    let start_today = PrimitiveDateTime::new(now.date(), time!(0:00)).assume_offset(now.offset());
    (start_today, now)
}
