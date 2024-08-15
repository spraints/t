use std::path::PathBuf;

use rocket::{fs::FileServer, get, routes};

#[derive(Clone)]
pub struct Options {
    pub static_root: PathBuf,
}

pub async fn web_main(opts: Options) {
    rocket::build()
        .manage(opts.clone())
        .mount("/", FileServer::from(&opts.static_root))
        .mount("/", routes![yay])
        .launch()
        .await
        .unwrap();
}

#[get("/yay")]
fn yay() -> &'static str {
    "yay\r\n"
}
