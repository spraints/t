pub mod real_time {
    use std::cell::RefCell;
    use time::{OffsetDateTime, UtcOffset};

    thread_local! {
        static LOCAL_OFFSET: RefCell<Option<UtcOffset>> = RefCell::new(None);
    }

    #[derive(Default, Clone)]
    pub struct DefaultTimeSource;

    impl super::TimeSource for DefaultTimeSource {
        fn now(&self) -> OffsetDateTime {
            OffsetDateTime::now_local()
        }

        fn local_offset(&self) -> UtcOffset {
            LOCAL_OFFSET.with(|cell| {
                let val = cell.borrow().as_ref().cloned();
                match val {
                    Some(ret) => ret,
                    None => {
                        let ret = UtcOffset::current_local_offset();
                        *cell.borrow_mut() = Some(ret);
                        ret
                    }
                }
            })
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
