use axum::{
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};

use crate::routes::{self, lobbies};

#[derive(Clone)]
pub(crate) struct CurrentUser {
    pub user_id: String,
}

// TODO(P1): Implement actual accounts.
// For now, we will pass the user_id (actually username) as plain text.
async fn auth_layer<B>(mut request: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let user_id = request
        .headers()
        .get("X-Auth-Token")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .to_string();

    request.extensions_mut().insert(CurrentUser { user_id });

    let response = next.run(request).await;
    Ok(response)
}

pub fn create_http_app() -> Router {
    Router::new()
        .route("/api/lobby/create", post(lobbies::create_lobby))
        .route("/api/lobby/join", post(lobbies::join_lobby))
        .route("/api/lobby/list", get(lobbies::list_lobbies))
        .route("/api/lobby/start", post(lobbies::start_lobby))
        // Layers act on only routes above.
        .route_layer(middleware::from_fn(auth_layer))
        .route("/", get(routes::root::hello))
}
