use std::path::PathBuf;

use rocket::fs::FileServer;
use rocket::serde::{json::Json, Serialize};
use rocket::{get, routes, State};
use t::file::t_open;

pub struct Options {
    pub static_root: PathBuf,
    pub t_data_file: PathBuf,
    pub time_source: TimeSource,
}

trait TS: t::timesource::TimeSource + Send + Sync {}

impl<TT: t::timesource::TimeSource + Send + Sync> TS for TT {}

pub struct TimeSource {
    ts: Box<dyn TS>,
}

impl TimeSource {
    pub fn new<T: 'static + t::timesource::TimeSource + Send + Sync>(ts: T) -> Self {
        Self { ts: Box::new(ts) }
    }
}

impl t::timesource::TimeSource for TimeSource {
    fn local_offset(&self) -> time::UtcOffset {
        self.ts.local_offset()
    }

    fn now(&self) -> time::OffsetDateTime {
        self.ts.now()
    }
}

pub async fn web_main(opts: Options) {
    let static_root = opts.static_root.clone();
    rocket::build()
        .manage(opts)
        .mount("/", FileServer::from(&static_root))
        .mount("/", routes![status])
        .launch()
        .await
        .unwrap();
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct Status {
    n: usize,
}

#[get("/api/status")]
fn status(opts: &State<Options>) -> Json<Status> {
    let tdf = t_open(&opts.t_data_file).unwrap();
    let es = tdf.read_entries(&opts.time_source).unwrap();
    Status { n: es.len() }.into()
}
