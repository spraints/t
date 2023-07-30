use t::entry::TimeEntry;
use t::parser::parse_time_entries;
use t::timesource::real_time::DefaultTimeSource;
use time::Date;
use time::Weekday::Sunday;

fn main() {
    let data_file = std::env::var("T_DATA_FILE").unwrap();
    let f = std::fs::File::open(data_file).unwrap();
    let entries = parse_time_entries(f, &DefaultTimeSource).unwrap();
    let mut counter = Counter {
        start: None,
        count: 0,
        highest: 0,
    };
    for entry in entries {
        counter = check(entry, counter);
    }
    counter.print();
    println!(
        "highest: {}",
        if counter.highest > counter.count {
            counter.highest
        } else {
            counter.count
        }
    );
}

struct Counter {
    start: Option<Date>,
    count: usize,
    highest: usize,
}

impl Counter {
    fn restart(mut self, date: Date) -> Self {
        if self.count > self.highest {
            self.highest = self.count;
        }
        self.start = Some(date);
        self.count = 1;
        self
    }

    fn incr(mut self) -> Self {
        self.count += 1;
        self
    }

    fn print(&self) {
        println!("{}: {}", self.start.unwrap(), self.count);
    }
}

fn check(entry: TimeEntry, counter: Counter) -> Counter {
    let sunday = sunday_for(entry);
    match counter.start {
        None => counter.restart(sunday),
        Some(date) => {
            if sunday != date {
                counter.print();
                counter.restart(sunday)
            } else {
                counter.incr()
            }
        }
    }
}

fn sunday_for(entry: TimeEntry) -> Date {
    start_of_week(entry.start_date())
}

fn start_of_week(dt: Date) -> Date {
    if dt.weekday() == Sunday {
        dt
    } else {
        start_of_week(dt.previous_day())
    }
}
