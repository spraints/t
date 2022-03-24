pub mod real_time {
    use std::cell::RefCell;
    use time::{OffsetDateTime, UtcOffset};

    thread_local! {
        static LOCAL_OFFSET: RefCell<Option<UtcOffset>> = RefCell::new(None);
    }

    #[derive(Default)]
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
    // Adapted from https://blog.iany.me/2019/03/how-to-mock-time-in-rust-tests-and-cargo-gotchas-we-met/

    use std::cell::RefCell;
    use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

    thread_local! {
        static MOCK_TIME: RefCell<Option<OffsetDateTime>> = RefCell::new(None);
    }

    #[derive(Default)]
    pub struct MockTimeSource /*{
        pub time: OffsetDateTime,
    }*/;

    impl super::TimeSource for MockTimeSource {
        fn now(&self) -> OffsetDateTime {
            //self.time.clone()
            MOCK_TIME.with(|cell| {
                cell.borrow()
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(OffsetDateTime::now_local)
            })
        }

        fn local_offset(&self) -> UtcOffset {
            self.now().offset()
        }
    }

    pub fn set_mock_time(date: Date, time: Time, offset: UtcOffset) {
        MOCK_TIME.with(|cell| {
            *cell.borrow_mut() = Some(PrimitiveDateTime::new(date, time).assume_offset(offset))
        });
    }
}

pub trait TimeSource {
    fn local_offset(&self) -> time::UtcOffset;
    fn now(&self) -> time::OffsetDateTime;
}
