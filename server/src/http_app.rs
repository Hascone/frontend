use axum::{routing::get, Router};

use crate::routes;

pub fn create_http_app() -> Router {
    Router::new().route("/", get(routes::root::hello))
}
