use crate::entry::Entry;
use time::{Date, Duration, Weekday::*};

pub fn each_week(entries: Vec<Entry>) -> DaysIterator {
    DaysIterator {
        entries,
        days: 7,
        last_date: None,
        next_index: 0,
    }
}

pub fn each_day(entries: Vec<Entry>) -> DaysIterator {
    DaysIterator {
        entries,
        days: 1,
        last_date: None,
        next_index: 0,
    }
}

pub struct DaysIterator {
    entries: Vec<Entry>,
    days: u8,
    last_date: Option<Date>,
    next_index: usize,
}

impl Iterator for DaysIterator {
    type Item = (Date, Vec<Entry>);

    fn next(&mut self) -> std::option::Option<Self::Item> {
        if self.next_index >= self.entries.len() {
            None
        } else {
            let date = match self.last_date {
                None => self.get_first_date(),
                Some(d) => d + self.span(),
            };
            let next_date = date + self.span();
            self.last_date = Some(date);
            let mut entries = vec![];
            for entry in self.entries.iter().skip(self.next_index) {
                if entry.start_date() >= next_date {
                    break;
                } else {
                    entries.push(entry.clone());
                    if entry.stop_date() >= next_date {
                        break;
                    }
                    self.next_index += 1;
                }
            }
            Some((date, entries))
        }
    }
}

impl DaysIterator {
    fn span(&self) -> Duration {
        Duration::days(self.days as i64)
    }

    fn get_first_date(&self) -> Date {
        match self.days {
            1 => self.entries[0].start_date(),
            7 => {
                let date = self.entries[0].start_date();
                match date.weekday() {
                    Sunday => date,
                    Monday => date - Duration::days(1),
                    Tuesday => date - Duration::days(2),
                    Wednesday => date - Duration::days(3),
                    Thursday => date - Duration::days(4),
                    Friday => date - Duration::days(5),
                    Saturday => date - Duration::days(6),
                }
            }
            x => panic!("Unable to iterate with span of {} days!", x),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_entries;
    use time::date;

    type TestRes = Result<(), Box<dyn std::error::Error>>;

    #[test]
    pub fn test_each_week_empty() {
        //let entries = parse_entries(
        let mut i = super::each_week(vec![]);
        assert_eq!(None, i.next());
    }

    #[test]
    pub fn test_each_day_empty() {
        let mut i = super::each_day(vec![]);
        assert_eq!(None, i.next());
    }

    #[test]
    pub fn test_each_week_one_week() -> TestRes {
        let entries = parse_entries(
            "2020-08-02 10:10,2020-08-02 11:10\n\
             2020-08-08 10:10,2020-08-08 11:10\n"
                .as_bytes(),
        )?;
        let mut i = super::each_week(entries.clone());
        assert_eq!(Some((date!(2020 - 08 - 02), entries)), i.next());
        assert_eq!(None, i.next());
        Ok(())
    }

    #[test]
    pub fn test_each_day_one_day() -> TestRes {
        let entries = parse_entries(
            "2020-08-02 10:10,2020-08-02 11:10\n\
             2020-08-02 10:10,2020-08-02 11:10\n"
                .as_bytes(),
        )?;
        let mut i = super::each_day(entries.clone());
        assert_eq!(Some((date!(2020 - 08 - 02), entries)), i.next());
        assert_eq!(None, i.next());
        Ok(())
    }

    #[test]
    pub fn test_each_week_two_weeks() -> TestRes {
        let mut entries = parse_entries(
            "2020-08-02 10:10,2020-08-02 11:10\n\
             2020-09-02 10:10,2020-09-02 11:10\n"
                .as_bytes(),
        )?;
        let mut i = super::each_week(entries.clone());
        assert_eq!(
            Some((date!(2020 - 08 - 02), entries.drain(..1).collect())),
            i.next()
        );
        assert_eq!(Some((date!(2020 - 08 - 09), vec![])), i.next());
        assert_eq!(Some((date!(2020 - 08 - 16), vec![])), i.next());
        assert_eq!(Some((date!(2020 - 08 - 23), vec![])), i.next());
        assert_eq!(Some((date!(2020 - 08 - 30), entries)), i.next());
        assert_eq!(None, i.next());
        Ok(())
    }

    #[test]
    pub fn test_each_day_two_days() -> TestRes {
        let mut entries = parse_entries(
            "2020-08-02 10:10,2020-08-02 11:10\n\
             2020-08-05 10:10,2020-08-05 11:10\n"
                .as_bytes(),
        )?;
        let mut i = super::each_day(entries.clone());
        assert_eq!(
            Some((date!(2020 - 08 - 02), entries.drain(..1).collect())),
            i.next()
        );
        assert_eq!(Some((date!(2020 - 08 - 03), vec![])), i.next());
        assert_eq!(Some((date!(2020 - 08 - 04), vec![])), i.next());
        assert_eq!(Some((date!(2020 - 08 - 05), entries)), i.next());
        assert_eq!(None, i.next());
        Ok(())
    }

    #[test]
    pub fn test_each_week_entry_spans_weeks() -> TestRes {
        let entries = parse_entries(
            "2020-08-02 10:10,2020-08-02 11:10\n\
             2020-08-08 10:10,2020-08-09 11:10\n\
             2020-09-02 10:10,2020-09-02 11:10\n"
                .as_bytes(),
        )?;
        let mut i = super::each_week(entries.clone());
        assert_eq!(
            Some((
                date!(2020 - 08 - 02),
                vec![entries[0].clone(), entries[1].clone()]
            )),
            i.next()
        );
        assert_eq!(
            Some((date!(2020 - 08 - 09), vec![entries[1].clone()])),
            i.next()
        );
        assert_eq!(Some((date!(2020 - 08 - 16), vec![])), i.next());
        assert_eq!(Some((date!(2020 - 08 - 23), vec![])), i.next());
        assert_eq!(
            Some((date!(2020 - 08 - 30), vec![entries[2].clone()])),
            i.next()
        );
        assert_eq!(None, i.next());
        Ok(())
    }

    #[test]
    pub fn test_each_day_entry_spans_days() -> TestRes {
        let entries = parse_entries(
            "2020-08-02 10:10,2020-08-02 11:10\n\
             2020-08-02 12:10,2020-08-03 11:10\n\
             2020-08-05 10:10,2020-08-05 11:10\n"
                .as_bytes(),
        )?;
        let mut i = super::each_day(entries.clone());
        assert_eq!(
            Some((
                date!(2020 - 08 - 02),
                vec![entries[0].clone(), entries[1].clone()]
            )),
            i.next()
        );
        assert_eq!(
            Some((date!(2020 - 08 - 03), vec![entries[1].clone()])),
            i.next()
        );
        assert_eq!(Some((date!(2020 - 08 - 04), vec![])), i.next());
        assert_eq!(
            Some((date!(2020 - 08 - 05), vec![entries[2].clone()])),
            i.next()
        );
        assert_eq!(None, i.next());
        Ok(())
    }

    #[test]
    pub fn test_each_week_first_entry_not_sunday() -> TestRes {
        let entries = parse_entries("2020-08-08 10:10,2020-08-08 11:10\n".as_bytes())?;
        let mut i = super::each_week(entries.clone());
        assert_eq!(Some((date!(2020 - 08 - 02), entries)), i.next());
        assert_eq!(None, i.next());
        Ok(())
    }
}
