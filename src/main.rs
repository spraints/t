use std::os::unix::process::CommandExt;
use t::extents;
use t::file::*;

fn main() {
    // Skip over the program name.
    let mut args = std::env::args().skip(1);
    match args.next() {
        None => usage(),
        Some(cmd) => match cmd.as_str() {
            "start" => cmd_start(args),
            "stop" => cmd_stop(args),
            "edit" => cmd_edit(args),
            "status" => cmd_status(args),
            "today" => cmd_today(args),
            "week" => cmd_week(args),
            "all" => (),
            "punchcard" => (),
            "days" => (),
            "csv" => (),
            "svg" => (),
            "pto" => (),
            "short" => (),
            "path" => (),
            "validate" => cmd_validate(),
            cmd => unknown_command(cmd),
        },
    }
}

fn unknown_command(cmd: &str) -> ! {
    eprintln!("Unsupported command: {}", cmd);
    std::process::exit(1)
}

fn usage() -> ! {
    eprintln!("A command (start, stop, edit) or query (status, today, week, all, punchcard, days, csv, svg, pto, short, path) is required.");
    std::process::exit(1)
}

fn cmd_start(_: impl Iterator) {
    cmd_validate();
    match start_new_entry().unwrap() {
        None => println!("Starting work."),
        Some(minutes) => println!("You already started working, {} minutes ago!", minutes),
    };
}

fn cmd_stop(_: impl Iterator) {
    cmd_validate();
    match stop_current_entry().unwrap() {
        Some(minutes) => println!("You just worked for {} minutes.", minutes),
        None => println!("You haven't started working yet!"),
    };
}

fn cmd_edit(_: impl Iterator) -> ! {
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

fn cmd_status(_: impl Iterator) {
    let entry = read_last_entry().expect(format!("error parsing {}", t_data_file().unwrap()).as_str());
    match entry {
        None => println!("NOT working"),
        Some(e) => match e.stop {
            None => println!("WORKING"),
            Some(_) => println!("NOT working"),
        },
    };
}

fn cmd_today(_: impl Iterator) {
    let (start_today, now) = extents::today();
    // longest week so far is 46 entries, so 100 should be totally fine for a day.
    let entries = read_last_entries(100).expect("error parsing data file");
    let minutes = entries.into_iter().fold(0, |sum, entry| {
        sum + entry.minutes_between(&start_today, &now)
    });
    println!("You have worked for {} minutes today.", minutes);
    println!("8h=480m");
}

fn cmd_week(_: impl Iterator) {
    let (start_week, now) = extents::this_week();
    // longest week so far is 46 entries, so 100 should be totally fine.
    let entries = read_last_entries(100).expect("error parsing data file");
    let minutes = entries.into_iter().fold(0, |sum, entry| {
        sum + entry.minutes_between(&start_week, &now)
    });
    println!(
        "You have worked for {} minutes since {}.",
        minutes,
        start_week.format("%Y-%m-%d")
    );
    println!("8h=480m 16h=960m 24h=1440m 32h=1920m 40h=2400m");
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
