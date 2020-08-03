use std::os::unix::process::CommandExt;

fn main() {
    // Skip over the program name.
    let mut args = std::env::args().skip(1);
    match args.next() {
        None => usage(),
        Some(cmd) => match cmd.as_str() {
            "start" => (),
            "stop" => (),
            "edit" => cmd_edit(args),
            "status" => (),
            "today" => (),
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
        }
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
    let path = std::env::var("T_DATA_FILE").unwrap();
    eprintln!("error: {}", std::process::Command::new(editor).arg(path).exec());
    std::process::exit(1)
}
