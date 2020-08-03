use std::os::unix::process::CommandExt;
use t::extents;
use t::file::*;

fn main() {
    // Skip over the program name.
    let mut args = std::env::args().skip(1);
    match args.next() {
        None => usage(),
        Some(cmd) => match cmd.as_str() {
            "start" => (),
            "stop" => (),
            "edit" => cmd_edit(args),
            "status" => cmd_status(args),
            "today" => cmd_today(args),
            "week" => (),
            "all" => (),
            "punchcard" => (),
            "days" => (),
            "csv" => (),
            "svg" => (),
            "pto" => (),
            "short" => (),
            "path" => (),
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

fn cmd_edit(_: impl Iterator) -> ! {
    let editor = std::env::var("EDITOR").unwrap();
    let path = t_data_file();
    eprintln!(
        "error: {}",
        std::process::Command::new(editor).arg(path).exec()
    );
    std::process::exit(1)
}

fn cmd_status(_: impl Iterator) {
    // TODO - only read the last entry!
    let entries = read_entries().expect("error parsing data file");
    match entries.last() {
        None => println!("NOT working"),
        Some(e) => match e.stop {
            None => println!("WORKING"),
            Some(_) => println!("NOT working"),
        },
    };
}

fn cmd_today(_: impl Iterator) {
    let (start_today, now) = extents::today();
    // TODO - only read the last 100 entries?
    let entries = read_entries().expect("error parsing data file");
    let minutes = entries.into_iter().fold(0, |sum, entry| {
        sum + entry.minutes_between(&start_today, &now)
    });
    println!("You have worked for {} minutes today.", minutes);
    println!("8h=480m");
}
