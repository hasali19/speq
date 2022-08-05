use std::net::SocketAddr;
use std::str::FromStr;

use axum::extract::Query;
use axum::response::IntoResponse;
use axum::Server;
use serde::Deserialize;
use speq::axum::get;
use speq::Reflect;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{:#?}", speq::spec());

    Server::bind(&SocketAddr::from_str("0.0.0.0:8000").unwrap())
        .serve(speq::axum::router().into_make_service())
        .await?;

    Ok(())
}

#[derive(Deserialize, Reflect)]
struct HelloWorldQuery {
    name: String,
}

#[get("/hello_world")]
async fn hello_world(#[query] query: Query<HelloWorldQuery>) -> impl IntoResponse {
    format!("hello, {}!", query.name)
}
