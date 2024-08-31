use std::path::PathBuf;

use rocket::data::ToByteUnit;
use rocket::fs::FileServer;
use rocket::serde::{json::Json, Serialize};
use rocket::{get, put, routes, Data, State};
use t::{extents, query};

pub struct Options {
    pub static_root: PathBuf,
    pub t_data_file: PathBuf,
    pub time_source: TimeSource,
}

pub fn main(opts: Options) {
    ::rocket::async_main(async { web_main(opts).await });
}

async fn web_main(opts: Options) {
    let static_root = opts.static_root.clone();
    rocket::build()
        .manage(opts)
        .mount("/", FileServer::from(&static_root))
        .mount("/", routes![status, upload])
        .launch()
        .await
        .unwrap();
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct Status {
    working: bool,
    last_update: Option<String>,
    minutes_today: i64,
    minutes_this_week: i64,
    recent: Vec<WeekStatus>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct WeekStatus {
    start_of_week: String,
    race_minutes: i64,
    total_minutes: i64,
}

#[get("/api/status")]
fn status(opts: &State<Options>) -> Result<Json<Status>, String> {
    let ctx = query::for_web(opts.t_data_file.clone(), &opts.time_source);
    let entries = ctx.all().or_else(|e| Err(format!("error: {e}")))?;
    Ok(Status {
        working: entries.is_working(),
        last_update: entries.last_update(),
        minutes_today: entries.minutes_between(extents::today()),
        minutes_this_week: entries.minutes_between(extents::this_week()),
        recent: entries
            .recent_weeks(4)
            .into_iter()
            .map(|w| WeekStatus {
                start_of_week: w.start.format("%Y-%m-%d"),
                race_minutes: w.minutes_to_date(),
                total_minutes: w.total_minutes(),
            })
            .collect(),
    }
    .into())
}

#[put("/api/t-data-file", data = "<body>")]
async fn upload(opts: &State<Options>, body: Data<'_>) -> std::io::Result<()> {
    body.open(128.mebibytes())
        .into_file(&opts.t_data_file)
        .await?;
    Ok(())
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

impl t::timesource::TimeSource for &TimeSource {
    fn local_offset(&self) -> time::UtcOffset {
        self.ts.local_offset()
    }

    fn now(&self) -> time::OffsetDateTime {
        self.ts.now()
    }
}
