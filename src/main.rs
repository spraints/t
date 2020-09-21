use gumdrop::Options;
use std::os::unix::process::CommandExt;
use t::entry::{local_offset, Entry};
use t::extents;
use t::file::*;
use t::iter::*;
use time::{Date, Duration, OffsetDateTime};

#[derive(Options)]
struct MainOptions {
    #[options(command)]
    command: Option<TCommand>,
}

#[derive(Options)]
enum TCommand {
    #[options(help = "start a time entry")]
    Start(NoArgs),
    #[options(help = "stop a time entry")]
    Stop(NoArgs),
    #[options(help = "edit the time entry database in $EDITOR")]
    Edit(NoArgs),
    #[options(help = "show current status")]
    Status(NoArgs),
    #[options(help = "show time worked today")]
    Today(NoArgs),
    #[options(help = "show time worked this week")]
    Week(NoArgs),
    #[options(help = "show spark graph of all entries")]
    All(NoArgs),
}

#[derive(Options)]
struct NoArgs {}

fn main() {
    let opts = MainOptions::parse_args_default_or_exit();
    match opts.command {
        None => usage(),
        Some(cmd) => match cmd {
            TCommand::Start(_) => cmd_start(),
            TCommand::Stop(_) => cmd_stop(),
            TCommand::Edit(_) => cmd_edit(),
            TCommand::Status(_) => cmd_status(),
            TCommand::Today(_) => cmd_today(),
            TCommand::Week(_) => cmd_week(),
            TCommand::All(_) => cmd_all(),
            //TCommand::Punchcard(_) => cmd_punchcard(),
            //TCommand::Days(_) => cmd_days(),
            //TCommand::CSV(_) => cmd_csv(),
            //TCommand::SVG(_) => cmd_svg(),
            //TCommand::PTO(_) => cmd_pto(),
            //TCommand::Short(_) => cmd_short(),
            //TCommand::Path(_) => cmd_path(),
            //TCommand::Validate(_) => cmd_validate(),
        },
    };
}

fn unknown_command(cmd: &str) -> ! {
    eprintln!("Unsupported command: {}", cmd);
    std::process::exit(1)
}

fn usage() -> ! {
    eprintln!("A command (start, stop, edit) or query (status, today, week, all, punchcard, days, csv, svg, pto, short, path) is required.");
    std::process::exit(1)
}

fn cmd_start() {
    cmd_validate();
    match start_new_entry().unwrap() {
        None => println!("Starting work."),
        Some(minutes) => println!("You already started working, {} minutes ago!", minutes),
    };
}

fn cmd_stop() {
    cmd_validate();
    match stop_current_entry().unwrap() {
        Some(minutes) => println!("You just worked for {} minutes.", minutes),
        None => println!("You haven't started working yet!"),
    };
}

fn cmd_edit() -> ! {
    let editor = std::env::var("EDITOR").unwrap();
    let path = t_data_file().unwrap();
    eprintln!(
        "error: {}",
        std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("{} \"$@\"", editor))
            .arg(editor)
            .arg(path)
            .exec()
    );
    std::process::exit(1)
}

fn cmd_status() {
    let entry =
        read_last_entry().expect(format!("error parsing {}", t_data_file().unwrap()).as_str());
    match entry {
        None => println!("NOT working"),
        Some(e) => match e.stop {
            None => println!("WORKING"),
            Some(_) => println!("NOT working"),
        },
    };
}

fn cmd_today() {
    let (start_today, now) = extents::today();
    // longest week so far is 46 entries, so 100 should be totally fine for a day.
    let entries = read_last_entries(100).expect("error parsing data file");
    let minutes = minutes_between(&entries, start_today, now);
    println!("You have worked for {} minutes today.", minutes);
    println!("8h=480m");
}

fn cmd_week() {
    let (start_week, now) = extents::this_week();
    // longest week so far is 46 entries, so 100 should be totally fine.
    let entries = read_last_entries(100).expect("error parsing data file");
    let minutes = minutes_between(&entries, start_week, now);
    println!(
        "You have worked for {} minutes since {}.",
        minutes,
        start_week.format("%Y-%m-%d")
    );
    println!("8h=480m 16h=960m 24h=1440m 32h=1920m 40h=2400m");
}

fn cmd_all() {
    // TODO - move all this to src/report.rs
    /*
    let width = match term_size::dimensions() {
        None => 80,
        Some((_, w)) => w,
    };
    */
    let entries = read_entries().unwrap();
    for (week_start, week_entries) in each_week(entries) {
        let week_end = week_start + Duration::days(6);
        let minutes = minutes_between_days(&week_entries, week_start, week_end);
        let segments = week_entries.len() as u32;
        let mut entry_minutes = week_entries.iter().map(|entry| entry.minutes());
        let (avg, min, max, stddev) = match segments {
            0 => (0, 0, 0, 0),
            1 => {
                let val = entry_minutes.next().unwrap_or(-1);
                (val, val, val, 0)
            }
            _ => {
                let avg = minutes / segments as i64; // lies
                let mut min = 10080;
                let mut max = 0;
                let mut sumsq: u64 = 0;
                for minutes in entry_minutes {
                    if minutes < min {
                        min = minutes
                    }
                    if minutes > max {
                        max = minutes
                    }
                    let diff = minutes - avg;
                    sumsq += (diff * diff) as u64;
                }
                (avg, min, max, sqrtint(sumsq / (segments as u64 - 1)))
            }
        };
        for (day, day_entries) in each_day(week_entries) {
            // make sparks!
        }
        println!("{week_start} - {week_end} {minutes:>6} min {segments:>4} segments  min/avg/max/stddev={min:>3}/{avg:>3}/{max:>3}/{stddev:>3}  {sparks}",
                 week_start=week_start,
                 week_end=week_end,
                 minutes=minutes,
                 segments=segments,
                 min=min,
                 avg=avg,
                 max=max,
                 stddev=stddev,
                 sparks=0);
    }
}

fn cmd_validate() {
    let mut maybe_last_entry = None;
    let mut n = 0;
    for entry in read_entries().unwrap() {
        n = n + 1;
        if let Err(err) = entry.is_valid_after(&maybe_last_entry) {
            println!("{}: {}", n, err);
        }
        maybe_last_entry = Some(entry);
    }
}

fn sqrtint(n: u64) -> u64 {
    let mut i = 1;
    while i * i <= n {
        i += 1;
    }
    i - 1
}

fn minutes_between(entries: &Vec<Entry>, start: OffsetDateTime, stop: OffsetDateTime) -> i64 {
    entries
        .iter()
        .fold(0, |sum, entry| sum + entry.minutes_between(start, stop))
}

fn minutes_between_days(entries: &Vec<Entry>, start: Date, stop: Date) -> i64 {
    minutes_between(
        entries,
        start.midnight().assume_offset(local_offset()),
        stop.next_day().midnight().assume_offset(local_offset()),
    )
}
