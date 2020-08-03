use time::{Duration, OffsetDateTime, Weekday::Sunday};

pub fn today() -> (OffsetDateTime, OffsetDateTime) {
    let now = OffsetDateTime::now_local();
    let start_today = now.date().midnight().assume_offset(now.offset());
    (start_today, now)
}

pub fn this_week() -> (OffsetDateTime, OffsetDateTime) {
    let now = OffsetDateTime::now_local();
    let start = start_of_week(&now);
    (start, now)
}

fn start_of_week(dt: &OffsetDateTime) -> OffsetDateTime {
    if dt.weekday() == Sunday {
        dt.date().midnight().assume_offset(dt.offset())
    } else {
        start_of_week(&(*dt - Duration::day()))
    }
}
