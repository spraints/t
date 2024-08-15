pub async fn web_main() {
    rocket::build().launch().await.unwrap();
}
