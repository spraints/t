use gumdrop::Options;
use std::os::unix::process::CommandExt;
use t::entry::Entry;
use t::extents;
use t::file::*;
use t::filter::filter_entries;
use t::report;
use time::{Duration, OffsetDateTime};

const DEFAULT_SPARKS: [char; 7] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇'];

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
    Status(StatusArgs),
    #[options(help = "generate output for bitbar")]
    Bitbar(BitBarArgs),
    #[options(help = "show time worked today")]
    Today(NoArgs),
    #[options(help = "show time worked this week")]
    Week(NoArgs),
    #[options(help = "show spark graph of all entries")]
    All(NoArgs),
    #[options(help = "show a table of time worked per day")]
    Days(DaysArgs),
    #[options(help = "show the path to t.csv")]
    Path(NoArgs),
    #[options(help = "check for any formatting errors in t.csv")]
    Validate(NoArgs),
}

#[derive(Options)]
struct StatusArgs {
    #[options(help = "also calculate the time worked this week so far")]
    with_week: bool,
}

#[derive(Options)]
struct BitBarArgs {
    #[options(help = "bitbar plugin script")]
    wrapper: String,
    #[options(help = "command to invoke")]
    command: String,
}

#[derive(Options)]
struct DaysArgs {
    #[options(free)]
    filters: Vec<String>,
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
            TCommand::Status(args) => {
                cmd_status(args);
            }
            TCommand::Bitbar(args) => cmd_bitbar(args),
            TCommand::Today(_) => cmd_today(),
            TCommand::Week(_) => cmd_week(),
            TCommand::All(_) => cmd_all(),
            //TCommand::Punchcard(_) => cmd_punchcard(),
            TCommand::Days(args) => cmd_days(args),
            //TCommand::CSV(_) => cmd_csv(),
            //TCommand::SVG(_) => cmd_svg(),
            //TCommand::PTO(_) => cmd_pto(),
            //TCommand::Short(_) => cmd_short(),
            TCommand::Path(_) => cmd_path(),
            TCommand::Validate(_) => cmd_validate(),
        },
    };
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
            .arg(format!("{} \"$@\"; t validate", editor))
            .arg(editor)
            .arg(path)
            .exec()
    );
    std::process::exit(1)
}

fn cmd_status(args: StatusArgs) -> bool {
    let entries = read_last_entries(100).expect("error parsing data file");
    let working = match entries.last() {
        None => false,
        Some(e) => e.stop.is_none(),
    };
    let status = if working { "WORKING" } else { "NOT working" };
    if args.with_week {
        let (start_week, now) = extents::this_week();
        let minutes = minutes_between(&entries, start_week, now);
        println!("{} ({})", status, minutes);
    } else {
        println!("{}", status);
    }
    working
}

fn cmd_bitbar(args: BitBarArgs) {
    match args.command.as_str() {
        "" => show_bitbar_plugin(&args.wrapper),
        "start" => cmd_start(),
        "stop" => cmd_stop(),
        _ => panic!("unrecognized command: {}", args.command),
    };
}

fn show_bitbar_plugin(mut wrapper: &str) {
    if wrapper == "" {
        wrapper = "t";
    }
    let working = cmd_status(StatusArgs { with_week: true });
    println!("---");
    if working {
        println!(
            "❚❚\tt stop | bash=\"{}\" param1=--command=stop terminal=false refresh=true",
            wrapper
        );
    } else {
        println!(
            "▶\tt start | bash=\"{}\" param1=--command=start terminal=false refresh=true",
            wrapper
        );
    }
}

fn cmd_today() {
    let (start_today, now) = extents::today();
    // longest week so far is 46 entries, so 100 should be totally fine for a day.
    let entries = read_last_entries(100).expect("error parsing data file");
    let minutes = minutes_between(&entries, start_today, now);
    println!("You have worked for {} minutes today.", minutes);
    print_day_legend();
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
    print_week_legend();
}

fn cmd_all() {
    let entries = read_entries().expect("error parsing data file");
    for line in report::all::calc(entries, &DEFAULT_SPARKS) {
        let week_end = line.start + Duration::days(6);
        print!("{} - {}   {:4} min", line.start, week_end, line.minutes);
        if let Some(analysis) = line.analysis {
            print!(
                " {:4} segments  min/avg/max/stddev={:3}/{:3}/{:3}/{:3}  ",
                line.segments, analysis.min, analysis.mean, analysis.max, analysis.stddev
            );
            let mut first = true;
            for day in analysis.sparks {
                if !day.is_empty() {
                    if !first {
                        print!("  ");
                    }
                    for spark in day {
                        print!("{}", spark);
                    }
                    first = false;
                }
            }
        }
        println!();
    }
    print_week_legend();
}

/*
let width = match term_size::dimensions() {
    None => 80,
    Some((_, w)) => w,
};
*/

fn cmd_days(args: DaysArgs) {
    let entries = read_entries().expect("error parsing data file");
    let entries = filter_entries(entries, args.filters).expect("unusable filter");
    print!("{}", report::days::prepare(entries));
    print_week_legend();
}

fn cmd_path() {
    println!("{}", t_data_file().unwrap());
}

fn cmd_validate() {
    let mut maybe_last_entry = None;
    for (n, entry) in read_entries().unwrap().into_iter().enumerate() {
        if let Err(err) = entry.is_valid_after(&maybe_last_entry) {
            println!("{}: {}", n, err);
        }
        maybe_last_entry = Some(entry);
    }
}

fn minutes_between(entries: &[Entry], start: OffsetDateTime, stop: OffsetDateTime) -> i64 {
    entries
        .iter()
        .fold(0, |sum, entry| sum + entry.minutes_between(start, stop))
}

fn print_day_legend() {
    println!("8h=480m");
}

fn print_week_legend() {
    println!("8h=480m 16h=960m 24h=1440m 32h=1920m 40h=2400m");
}
