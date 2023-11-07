#[macro_use]
extern crate lazy_static;

mod http_app;
mod routes;
mod sockets;

use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;

async fn setup_http() {
    let app = http_app::create_http_app();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("Spinning up HTTP server");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failed to find HTTP server.");
}

async fn setup_ws() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    let socket = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind websocket.");

    println!("Spinning up Websocket server");

    while let Ok((stream, addr)) = socket.accept().await {
        tokio::spawn(sockets::handle_connection(stream, addr));
    }
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    tokio::join!(setup_http(), setup_ws());
}
