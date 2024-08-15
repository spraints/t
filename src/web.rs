use rocket::{get, routes};

pub async fn web_main() {
    rocket::build()
        .mount("/", routes![yay])
        .launch()
        .await
        .unwrap();
}

#[get("/")]
fn yay() -> &'static str {
    "yay\r\n"
}
