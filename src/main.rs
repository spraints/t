fn main() {
    // Skip over the program name.
    let mut args = std::env::args().skip(1);
    match args.next() {
        None => usage(),
        Some(cmd) => match cmd.as_str() {
            "start" => (),
            "stop" => (),
            "edit" => (),
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
