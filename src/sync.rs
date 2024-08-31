use std::path::PathBuf;

pub struct Options {
    pub url: String,
    pub verbose: bool,
    pub log_file: Option<PathBuf>,
}

pub fn main(opts: Options) {
    let Options {
        url,
        verbose,
        log_file,
    } = opts;
    todo!("sync to {url}, verbose={verbose}, logging to {log_file:?}");
}
