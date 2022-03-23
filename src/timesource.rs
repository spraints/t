#[cfg(not(test))]
pub mod real_time {
    use std::cell::RefCell;
    use time::{OffsetDateTime, UtcOffset};

    thread_local! {
        static LOCAL_OFFSET: RefCell<Option<UtcOffset>> = RefCell::new(None);
    }

    pub fn now() -> OffsetDateTime {
        OffsetDateTime::now_local()
    }

    pub fn local_offset() -> UtcOffset {
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

#[cfg(test)]
pub mod mock_time {
    // Adapted from https://blog.iany.me/2019/03/how-to-mock-time-in-rust-tests-and-cargo-gotchas-we-met/

    use std::cell::RefCell;
    use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

    thread_local! {
        static MOCK_TIME: RefCell<Option<OffsetDateTime>> = RefCell::new(None);
    }

    pub fn now() -> OffsetDateTime {
        //self.time.clone()
        MOCK_TIME.with(|cell| {
            cell.borrow()
                .as_ref()
                .cloned()
                .unwrap_or_else(OffsetDateTime::now_local)
        })
    }

    pub fn local_offset() -> UtcOffset {
        now().offset()
    }

    pub fn set_mock_time(date: Date, time: Time, offset: UtcOffset) {
        MOCK_TIME.with(|cell| {
            *cell.borrow_mut() = Some(PrimitiveDateTime::new(date, time).assume_offset(offset))
        });
    }
}

#[cfg(test)]
pub use mock_time::{local_offset, now};
#[cfg(not(test))]
pub use real_time::{local_offset, now};
