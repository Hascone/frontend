use axum::{http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

use crate::{
    http_app::CurrentUser,
    sockets::broadcast_to_lobby,
    state::lobby::{Lobby, LobbyState, LOBBIES},
};

#[derive(Serialize)]
pub(crate) struct CreateLobbyResponse {
    code: String,
}

pub(crate) async fn create_lobby(
    Extension(current_user): Extension<CurrentUser>,
) -> (StatusCode, Json<CreateLobbyResponse>) {
    // TODO(P0): Randomize ID.
    let code = "123".to_string();

    let lobby = Lobby {
        code: code.clone(),
        user_ids: vec![current_user.user_id.clone()],
        state: LobbyState::Waiting,
    };

    LOBBIES.lock().unwrap().insert(lobby.code.clone(), lobby);

    (StatusCode::CREATED, Json(CreateLobbyResponse { code }))
}

#[derive(Deserialize)]
pub(crate) struct JoinLobbyRequest {
    code: String,
}

pub(crate) async fn join_lobby(
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<JoinLobbyRequest>,
) -> Result<StatusCode, StatusCode> {
    let mut lobbies = LOBBIES.lock().unwrap();
    let lobby = lobbies
        .get_mut(&request.code)
        .ok_or(StatusCode::NOT_FOUND)?;

    if lobby.user_ids.contains(&current_user.user_id) {
        return Err(StatusCode::BAD_REQUEST);
    }

    lobby.user_ids.push(current_user.user_id.clone());
    broadcast_to_lobby(
        &lobby,
        Some(&current_user.user_id),
        // TODO: Make an actual websocket message. Like in JSON.
        Message::Text("Lobby joined".to_owned()),
    );

    Ok(StatusCode::OK)
}

#[derive(Serialize)]
pub(crate) struct ListLobbiesResponse {
    lobbies: Vec<Lobby>,
}

pub(crate) async fn list_lobbies() -> (StatusCode, Json<ListLobbiesResponse>) {
    let result: Vec<Lobby> = LOBBIES
        .lock()
        .unwrap()
        .values()
        .filter(|lobby| matches!(lobby.state, LobbyState::Waiting))
        .cloned()
        .collect();

    (
        StatusCode::OK,
        Json(ListLobbiesResponse { lobbies: result }),
    )
}

#[derive(Deserialize)]
pub(crate) struct StartLobbyRequest {
    code: String,
}

pub(crate) async fn start_lobby(
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<StartLobbyRequest>,
) -> Result<StatusCode, StatusCode> {
    let mut lobbies = LOBBIES.lock().unwrap();
    let lobby = lobbies
        .get_mut(&request.code)
        .ok_or(StatusCode::NOT_FOUND)?;

    if !lobby.user_ids.contains(&current_user.user_id) {
        return Err(StatusCode::BAD_REQUEST);
    }

    lobby.state = LobbyState::InProgress;

    Ok(StatusCode::OK)
}
