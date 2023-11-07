use std::{collections::HashMap, sync::Mutex};

use serde::Serialize;

#[derive(Serialize, Clone)]
pub enum LobbyState {
    Waiting = 0,
    InProgress = 1,
    Finished = 2,
}

#[derive(Serialize, Clone)]
pub struct Lobby {
    pub code: String,
    pub user_ids: Vec<String>,
    pub state: LobbyState,
}

lazy_static! {
    // TODO: Maybe we should abstract this away in the future.
    pub(crate) static ref LOBBIES: Mutex<HashMap<String, Lobby>> = Mutex::new(HashMap::new());
}
