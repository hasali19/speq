use std::net::SocketAddr;
use std::str::FromStr;

use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Server;
use serde::Deserialize;
use speq::axum::get;
use speq::Reflect;

speq::axum_config!(i32);

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{:#?}", speq::spec());

    Server::bind(&SocketAddr::from_str("0.0.0.0:8000").unwrap())
        .serve(speq::axum_router!().with_state(42).into_make_service())
        .await?;

    Ok(())
}

#[derive(Deserialize, Reflect)]
struct HelloWorldQuery {
    name: String,
}

#[get("/hello_world")]
async fn hello_world(
    #[query] query: Query<HelloWorldQuery>,
    State(state): State<i32>,
) -> impl IntoResponse {
    format!("hello, {}! state={state}", query.name)
}
