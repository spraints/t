mod sync;
mod web;

use gumdrop::Options;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::process::CommandExt;
use t::entry::into_time_entries;
use t::entry::Entry;
use t::entry::TimeEntry;
use t::extents;
use t::file::*;
use t::filter::filter_entries;
use t::query::{self, EntriesResult};
use t::report;
use t::timesource::real_time::DefaultTimeSource;
use time::{Duration, OffsetDateTime};

const DEFAULT_SPARKS: [char; 7] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇'];

const HOUR_EMOJI: [char; 12] = [
    '🕛', '🕐', '🕑', '🕒', '🕓', '🕔', '🕕', '🕖', '🕗', '🕘', '🕙', '🕚',
];
const CHECK_EMOJI: char = '✅';

const FULL_WEEK: i64 = 5 * 8 * 60; // 5 day, 8 hours per day, 60 minutes per hour.
const MY_FULL_WEEK: i64 = 2000; // This is my goal.

#[derive(Options)]
struct MainOptions {
    #[options(command)]
    command: Option<TCommand>,

    #[options(help = "show this help message")]
    help: bool,
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
    #[options(help = "compare my current progress this week against previous weeks")]
    Race(RaceArgs),
    #[options(help = "show spark graph of all entries")]
    All(NoArgs),
    #[options(help = "show a table of time worked per day")]
    Days(DaysArgs),
    #[options(help = "produce a CSV report (see help for options)")]
    Csv(CSVArgs),
    #[options(
        help = "show the amount of time off I took per year, with optional number of minutes per full time week"
    )]
    Pto(PtoArgs),
    #[options(help = "show the path to t.csv")]
    Path(NoArgs),
    #[options(help = "check for any formatting errors in t.csv")]
    Validate(ValidateArgs),
    #[options(help = "show current timestamp as it would be written to t.csv")]
    Now(NoArgs),
    #[options(help = "show all annotations in t.csv")]
    Notes(NoArgs),
    #[options(help = "run a web server")]
    Web(WebArgs),
    #[options(help = "sync to a web server")]
    Sync(SyncArgs),
}

#[derive(Options)]
struct StatusArgs {
    #[options(help = "also calculate the time worked this week so far")]
    with_week: bool,
    #[options(help = "also list entries")]
    list: Option<StatusTimePeriod>,
    #[options(help = "show this message")]
    help: bool,
}

enum StatusTimePeriod {
    Today,
    Week,
}

impl std::str::FromStr for StatusTimePeriod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "t" | "day" | "today" => Ok(Self::Today),
            "w" | "week" => Ok(Self::Week),
            _ => Err(format!("expected 'today' or 'week' but got {s:?}")),
        }
    }
}

#[derive(Options)]
struct BitBarArgs {
    #[options(help = "bitbar plugin script")]
    wrapper: String,
    #[options(help = "command to invoke")]
    command: String,
    #[options(help = "show this message")]
    help: bool,
}

#[derive(Options)]
struct DaysArgs {
    #[options(help = "include monthly and yearly totals")]
    summary: bool,
    #[options(help = "only show yearly totals")]
    annual: bool,
    #[options(
        free,
        help = "A year (YYYY) or month (YYYY-MM) to show (default is all)"
    )]
    filters: Vec<String>,
    #[options(help = "show this message")]
    help: bool,
}

#[derive(Options, Default)]
struct ValidateArgs {
    #[options(help = "Number of times to validate (useful for benchmarking)")]
    count: Option<usize>,
    #[options(help = "Show some extra information about the file")]
    verbose: bool,
}

impl From<DaysArgs> for (Vec<String>, report::days::Options) {
    fn from(val: DaysArgs) -> Self {
        (
            val.filters,
            report::days::Options {
                include_totals: val.summary,
                only_show_per_year: val.annual,
            },
        )
    }
}

#[derive(Options)]
struct RaceArgs {
    #[options(help = "number of previous weeks to consider")]
    count: Option<i16>,
    #[options(help = "show this message")]
    help: bool,
}

#[derive(Options)]
struct CSVArgs {
    #[options(
        free,
        parse(try_from_str = "ReportType::try_parse"),
        help = "type of CSV report to create"
    )]
    report_type: Option<ReportType>,

    #[options(help = "show this message")]
    help: bool,
}

enum ReportType {
    Weekly,
    YearVsYear,
}

impl ReportType {
    fn try_parse(arg: &str) -> Result<Self, String> {
        match arg {
            "weekly" | "weeks" | "w" => Ok(Self::Weekly),
            "yvy" | "year-vs-year" => Ok(Self::YearVsYear),
            _ => Err(format!("unrecognized report type {arg:?}")),
        }
    }
}

#[derive(Options)]
struct PtoArgs {
    #[options(free)]
    full_week: Option<i64>,
    #[options(help = "show this message")]
    help: bool,
}

#[derive(Options)]
struct WebArgs {
    #[options(help = "path to static files")]
    static_path: Option<String>,

    #[options(help = "path to t.csv (default: T_DATA_FILE or t.csv)")]
    t_data_file: Option<String>,

    #[options(help = "show this message")]
    help: bool,
}

impl From<WebArgs> for web::Options {
    fn from(val: WebArgs) -> Self {
        let static_root = match val.static_path {
            None => "public".into(),
            Some(path) => path.into(),
        };
        let t_data_file = val
            .t_data_file
            .or_else(|| t::file::t_data_file().ok())
            .unwrap_or_else(|| "t.csv".to_string())
            .into();
        web::Options {
            static_root,
            t_data_file,
            time_source: web::TimeSource::new(TIME_SOURCE.clone()),
        }
    }
}

#[derive(Options)]
struct SyncArgs {
    #[options(free, help = "base URL where data should be synced")]
    url: Option<String>,

    #[options(help = "verbose")]
    verbose: bool,

    #[options(help = "where to write output")]
    log_file: Option<String>,

    #[options(help = "show this message")]
    help: bool,
}

impl TryInto<sync::Options> for SyncArgs {
    type Error = &'static str;

    fn try_into(self) -> Result<sync::Options, Self::Error> {
        match self.url {
            None => Err("url must be provided"),
            Some(url) => Ok(sync::Options {
                url,
                verbose: self.verbose,
                log_file: self.log_file.map(|f| f.into()),
            }),
        }
    }
}

#[derive(Options)]
struct NoArgs {
    #[options(help = "show this message")]
    help: bool,
}

static TIME_SOURCE: DefaultTimeSource = DefaultTimeSource;

fn main() {
    let opts = MainOptions::parse_args_default_or_exit();
    match opts.command {
        None => usage(),
        Some(cmd) => match cmd {
            TCommand::Start(_) => cmd_start(),
            TCommand::Stop(_) => cmd_stop(),
            TCommand::Edit(_) => cmd_edit(),
            TCommand::Status(args) => cmd_status(args),
            TCommand::Bitbar(args) => cmd_bitbar(args),
            TCommand::Today(_) => cmd_today(),
            TCommand::Week(_) => cmd_week(),
            TCommand::Race(args) => cmd_race(args),
            TCommand::All(_) => cmd_all(),
            //TCommand::Punchcard(_) => cmd_punchcard(),
            TCommand::Days(args) => cmd_days(args),
            TCommand::Csv(args) => cmd_csv(args),
            //TCommand::SVG(_) => cmd_svg(),
            TCommand::Pto(args) => cmd_pto(args),
            //TCommand::Short(_) => cmd_short(),
            TCommand::Path(_) => cmd_path(),
            TCommand::Validate(args) => cmd_validate(args),
            TCommand::Now(_) => cmd_now(),
            TCommand::Notes(_) => cmd_notes(),
            TCommand::Web(args) => web::main(args.into()),
            TCommand::Sync(args) => sync::main(gentle_unwrap(args.try_into())),
        },
    };
}

fn gentle_unwrap<T, E: Display>(r: Result<T, E>) -> T {
    match r {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

fn usage() -> ! {
    eprintln!("A command (start, stop, edit) or query (status, today, week, all, punchcard, days, csv, svg, pto, short, path) is required.");
    std::process::exit(1)
}

fn cmd_start() {
    cmd_validate(Default::default());
    match start_new_entry(&TIME_SOURCE).unwrap() {
        None => println!("Starting work."),
        Some(minutes) => println!("You already started working, {} minutes ago!", minutes),
    };
}

fn cmd_stop() {
    cmd_validate(Default::default());
    match stop_current_entry(&TIME_SOURCE).unwrap() {
        Some((true, minutes)) => println!("You just worked for {} minutes.", minutes),
        Some((false, minutes)) => println!("You stopped {} minutes ago.", minutes),
        None => println!("You haven't started working yet!"),
    };
}

fn cmd_edit() -> ! {
    let editor = std::env::var("EDITOR").unwrap();
    let path = t_data_file().unwrap();
    let t = std::env::current_exe().unwrap();

    // If we're using a vi-like editor, tell it to jump to the end of the file.
    let args = if editor.split("/").last().unwrap().contains("vi") {
        match File::open(&path) {
            Ok(f) => {
                let line_count = BufReader::new(f).lines().count();
                format!("+{line_count}")
            }
            _ => "".to_owned(),
        }
    } else {
        "".to_owned()
    };

    let cmd = format!("{editor} {args} \"$@\"; {t:?} validate");
    eprintln!(
        "error: {}",
        std::process::Command::new("sh")
            .arg("-xc")
            .arg(cmd)
            .arg(editor)
            .arg(path)
            .exec()
    );
    std::process::exit(1)
}

fn cmd_status(args: StatusArgs) {
    let ui: CLIStatusUI = args.into();
    show_status(ui);
}

fn show_status(ui: impl StatusUI) -> bool {
    let entries = query::for_cli(TIME_SOURCE.clone())
        .tail()
        .expect("error parsing data file");
    println!("{}", ui.format(&entries));
    entries.is_working()
}

trait StatusUI {
    fn format(&self, entries: &EntriesResult) -> String;
}

struct CLIStatusUI {
    with_week: bool,
    list: Option<StatusTimePeriod>,
}

impl StatusUI for CLIStatusUI {
    fn format(&self, entries: &EntriesResult) -> String {
        let status_str = if entries.is_working() {
            "WORKING"
        } else {
            "NOT working"
        };
        let list = match self.list {
            None => "".into(),
            Some(StatusTimePeriod::Today) => self.list_entries(entries.between(extents::today())),
            Some(StatusTimePeriod::Week) => {
                self.list_entries(entries.between(extents::this_week()))
            }
        };
        if self.with_week {
            let minutes = entries.minutes_between(extents::this_week());
            format!("{status_str} ({minutes}){list}")
        } else {
            format!("{status_str}{list}")
        }
    }
}

impl CLIStatusUI {
    fn list_entries(&self, entries: Vec<TimeEntry>) -> std::borrow::Cow<str> {
        if entries.is_empty() {
            return "\n(no entries)".into();
        }
        let mut cur_date = None;
        let mut res = "".to_string();
        for e in entries {
            let ed = e.start.date();
            if Some(ed) != cur_date {
                res.push_str(&format!("\n\n{ed}:"));
                cur_date = Some(ed);
            }
            res.push_str(&format!("\n  {}", e.start.time()));
            match e.stop {
                None => res.push_str(" - (still working)"),
                Some(t) => res.push_str(&format!(" - {}", t.time())),
            };
        }
        res.into()
    }
}

impl From<StatusArgs> for CLIStatusUI {
    fn from(args: StatusArgs) -> Self {
        CLIStatusUI {
            with_week: args.with_week,
            list: args.list,
        }
    }
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
    if wrapper.is_empty() {
        wrapper = "t";
    }
    let working = show_status(BitBarStatusUI);
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
    println!("---");
    show_entry();
    show_today();
    show_week();
    println!("---");
    show_race(4, " | font=Monaco");
}

fn show_entry() {
    let entries = read_last_entries(1, &TIME_SOURCE).expect("error parsing data file");
    match entries.last() {
        Some(Entry::Time(te)) if !te.is_finished() => {
            println!("Working for {} minutes.", te.minutes(&TIME_SOURCE))
        }
        Some(Entry::Note(n)) => println!("{n}"),
        _ => println!("Not working."),
    }
}

struct BitBarStatusUI;

impl StatusUI for BitBarStatusUI {
    fn format(&self, entries: &EntriesResult) -> String {
        let status_str = if entries.is_working() { "👔" } else { "😴" };
        let minutes = entries.minutes_between(extents::this_week());
        format!("{status_str}{}", week_progress_emoji(minutes))
    }
}

fn cmd_today() {
    show_today();
    print_day_legend();
}

fn show_today() {
    let (start_today, now) = extents::today();
    // longest week so far is 46 entries, so 100 should be totally fine for a day.
    let entries = read_last_entries(100, &TIME_SOURCE).expect("error parsing data file");
    let entries = into_time_entries(entries);
    let minutes = minutes_between(&entries, start_today, now);
    println!("You have worked for {} minutes today.", minutes);
}

fn cmd_week() {
    show_week();
    print_week_legend();
}

fn show_week() {
    let (start_week, now) = extents::this_week();
    // longest week so far is 46 entries, so 100 should be totally fine.
    let entries = read_last_entries(100, &TIME_SOURCE).expect("error parsing data file");
    let entries = into_time_entries(entries);
    let minutes = minutes_between(&entries, start_week, now);
    println!(
        "You have worked for {} minutes since {}.",
        minutes,
        start_week.format("%Y-%m-%d")
    );
}

fn cmd_race(args: RaceArgs) {
    let RaceArgs { count, help: _ } = args;
    show_race(count.unwrap_or(1), "");
}

fn show_race(previous_weeks: i16, suffix: &str) {
    let res = query::for_cli(TIME_SOURCE.clone())
        .all()
        .expect("error parsing data file");
    let (start_week, now) = extents::this_week();
    let minutes_this_week = res.minutes_between((start_week, now));

    let mut total_prev_minutes = 0;
    let mut behind = 0;
    let mut ahead = 0;
    for w in res.recent_weeks(previous_weeks) {
        let minutes = w.minutes_to_date();
        println!(
            "{}: {} {:4} minutes {}{}",
            w.start.format("%Y-%m-%d"),
            week_progress_emoji(minutes),
            minutes,
            race_bars(minutes),
            suffix,
        );
        total_prev_minutes += minutes;
        if minutes_this_week > minutes {
            ahead += 1;
        } else {
            behind += 1;
        }
    }

    let summary = match (previous_weeks, total_prev_minutes, minutes_this_week) {
        (1, prev, cur) if prev == cur => "equal!".to_string(),
        (1, prev, cur) if prev < cur => "ahead of last week!".to_string(),
        (1, _, _) => "behind last week".to_string(),
        (c, prev, cur) => format!(
            "ahead of {}, behind {}, avg {:+}",
            ahead,
            behind,
            cur - (prev / c as i64)
        ),
    };
    println!(
        "{}: {} {:4} minutes {}{}",
        start_week.format("%Y-%m-%d"),
        week_progress_emoji(minutes_this_week),
        minutes_this_week,
        race_bars(minutes_this_week),
        suffix,
    );
    println!("{}", summary);
}

fn race_bars(n: i64) -> String {
    let bar_count = n / 60;
    if bar_count > 40 {
        "▇".repeat(40) + "..."
    } else {
        "▇".repeat(bar_count as usize)
    }
}

fn cmd_all() {
    let entries = read_time_entries(&TIME_SOURCE).expect("error parsing data file");
    for line in report::all::calc(entries, &DEFAULT_SPARKS, &TIME_SOURCE) {
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
    let (filters, opts) = args.into();

    let entries = read_time_entries(&TIME_SOURCE).expect("error parsing data file");
    let entries = filter_entries(entries, filters).expect("unusable filter");

    print!("{}", report::days::prepare(entries, &TIME_SOURCE, opts));
    print_week_legend();
}

fn cmd_csv(args: CSVArgs) {
    match args.report_type {
        None => eprintln!("report type is required"),
        Some(ReportType::Weekly) => {
            let entries = read_time_entries(&TIME_SOURCE).expect("error parsing data file");
            println!("start of week,minutes");
            for line in report::all::calc(entries, &DEFAULT_SPARKS, &TIME_SOURCE) {
                println!("{},{}", line.start, line.minutes);
            }
        }
        Some(ReportType::YearVsYear) => {
            let entries = read_time_entries(&TIME_SOURCE).expect("error parsing data file");
            let mut years = Vec::new();
            struct Year {
                year: i32,
                weeks: HashMap<u8, i64>,
            }
            let mut cur_year: Option<Year> = None;
            for line in report::all::calc(entries, &DEFAULT_SPARKS, &TIME_SOURCE) {
                let mut yy = match (line.start.year(), cur_year) {
                    (y, Some(yy)) if y == yy.year => yy,
                    (_, None) => Year {
                        year: line.start.year(),
                        weeks: HashMap::new(),
                    },
                    (_, Some(yy)) => {
                        years.push(yy);
                        Year {
                            year: line.start.year(),
                            weeks: HashMap::new(),
                        }
                    }
                };
                yy.weeks.insert(line.start.week(), line.minutes);
                cur_year = Some(yy);
            }
            if let Some(yy) = cur_year.take() {
                years.push(yy);
            }
            print!("week of year");
            for yy in &years {
                print!(",{}", yy.year);
            }
            println!();
            for week_num in 1..=53 {
                print!("{week_num}");
                for yy in &years {
                    match yy.weeks.get(&week_num) {
                        None => print!(","),
                        Some(min) => print!(",{min}"),
                    };
                }
                println!();
            }
        }
    };
}

fn cmd_pto(args: PtoArgs) {
    let entries = read_time_entries(&TIME_SOURCE).expect("error parsing data file");
    let full_week = args.full_week.unwrap_or(FULL_WEEK);
    print!("{}", report::pto::prepare(entries, full_week, &TIME_SOURCE));
    print_week_legend();
}

fn cmd_path() {
    println!("{}", t_data_file().unwrap());
}

fn cmd_validate(args: ValidateArgs) {
    let ValidateArgs { count, verbose } = args;
    for i in 0..count.unwrap_or(1) {
        do_validate(verbose && i == 0);
    }
}

fn do_validate(verbose: bool) {
    let mut last_time_entry = None;
    let mut last_entry_is_finished = true;
    let mut count = 0;
    for (n, entry) in read_entries(&TIME_SOURCE).unwrap().into_iter().enumerate() {
        if !last_entry_is_finished {
            println!("{}: previous entry is not finished", n);
            last_entry_is_finished = true;
        }
        if let Entry::Time(te) = entry {
            if let Err(err) = te.is_valid_after(&last_time_entry) {
                println!("{}: {}", n, err);
            }
            last_time_entry = Some(te);
        }
        count += 1;
    }
    if verbose {
        println!("Checked {count} lines.");
    }
}

fn cmd_now() {
    let t = t::entry::Time::at(t::timesource::TimeSource::now(&TIME_SOURCE));
    println!("{}", t);
}

fn cmd_notes() {
    let mut last_time = None;
    for entry in read_entries(&TIME_SOURCE).unwrap().into_iter() {
        match entry {
            Entry::Note(s) => println!("{}: {}", last_time.as_ref().unwrap(), s.trim()),
            Entry::Time(te) => last_time = Some(format!("{}", te.start)),
        };
    }
}

fn minutes_between(entries: &[TimeEntry], start: OffsetDateTime, stop: OffsetDateTime) -> i64 {
    entries
        .iter()
        .fold(0, |sum, entry| sum + entry.minutes_between(start, stop))
}

fn week_progress_emoji(minutes: i64) -> char {
    let fraction = HOUR_EMOJI.len() * minutes as usize / MY_FULL_WEEK as usize;
    HOUR_EMOJI.get(fraction).copied().unwrap_or(CHECK_EMOJI)
}

fn print_day_legend() {
    println!("8h=480m");
}

fn print_week_legend() {
    println!("8h=480m 16h=960m 24h=1440m 32h=1920m 40h=2400m");
}
