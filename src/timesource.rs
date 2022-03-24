pub mod real_time {
    use time::{OffsetDateTime, UtcOffset};

    #[derive(Clone)]
    pub struct DefaultTimeSource;

    impl super::TimeSource for DefaultTimeSource {
        fn now(&self) -> OffsetDateTime {
            OffsetDateTime::now_local()
        }

        fn local_offset(&self) -> UtcOffset {
            UtcOffset::current_local_offset()
        }
    }
}

#[cfg(test)]
pub mod mock_time {
    use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

    #[derive(Clone)]
    pub struct MockTimeSource {
        dt: OffsetDateTime,
    }

    impl super::TimeSource for MockTimeSource {
        fn now(&self) -> OffsetDateTime {
            self.dt.clone()
        }

        fn local_offset(&self) -> UtcOffset {
            self.dt.offset()
        }
    }

    pub fn mock_time(date: Date, time: Time, offset: UtcOffset) -> MockTimeSource {
        MockTimeSource {
            dt: PrimitiveDateTime::new(date, time).assume_offset(offset),
        }
    }
}

pub trait TimeSource {
    fn local_offset(&self) -> time::UtcOffset;
    fn now(&self) -> time::OffsetDateTime;
}
