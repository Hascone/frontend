#[macro_use]
extern crate lazy_static;

mod http_app;
mod routes;
mod sockets;
mod state;

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

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    tokio::join!(setup_http(), setup_ws());
}
