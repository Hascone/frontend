use std::{collections::HashMap, net::SocketAddr, sync::Mutex};

use bimap::BiMap;
use futures_util::{
    future::{ok, select},
    pin_mut, StreamExt, TryStreamExt,
};
use serde::Deserialize;
use tokio::{
    net::TcpStream,
    sync::mpsc::{unbounded_channel, UnboundedSender},
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_tungstenite::tungstenite::Message;

use crate::state::lobby::Lobby;

lazy_static! {
    static ref CONNECTIONS: Mutex<HashMap<SocketAddr, UnboundedSender<Message>>> =
        Mutex::new(HashMap::new());
    static ref USERS_TO_ADDRESSES: Mutex<BiMap<String, SocketAddr>> = Mutex::new(BiMap::new());
}

#[derive(Deserialize)]
struct IdentifyMessage {
    user_id: String,
}

pub async fn handle_connection(stream: TcpStream, addr: SocketAddr) {
    println!("New connection: {:?}", addr);

    let ws = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during websocket handshake occurred.");

    let (tx, rx) = unbounded_channel();
    CONNECTIONS.lock().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws.split();

    // For now, broadcast the message to everyone but ourselves.
    let incoming_stream = incoming.try_for_each(|msg| {
        println!("Received message from {}", addr);

        if let Message::Text(data) = msg {
            let identify = serde_json::from_str::<IdentifyMessage>(&data);
            if let Ok(data) = identify {
                USERS_TO_ADDRESSES
                    .lock()
                    .unwrap()
                    .insert(data.user_id.clone(), addr);
            }
        } else {
            let peers = CONNECTIONS.lock().unwrap();
            let recipients = peers
                .iter()
                .filter(|(peer_addr, _)| peer_addr != &&addr)
                .map(|(_, ws_sink)| ws_sink);

            recipients.for_each(|ws_sink| {
                ws_sink.send(msg.clone()).unwrap();
            });
        }

        ok(())
    });

    let rx_stream = UnboundedReceiverStream::new(rx);
    let receive_from_others = rx_stream.map(Ok).forward(outgoing);

    pin_mut!(incoming_stream, receive_from_others);
    select(incoming_stream, receive_from_others).await;

    println!("{} disconnected", addr);
    CONNECTIONS.lock().unwrap().remove(&addr);
}

pub fn broadcast_to_lobby(lobby: &Lobby, exclude_user_id: Option<&str>, message: Message) {
    let addr_map = USERS_TO_ADDRESSES.lock().unwrap();
    let connection_map = CONNECTIONS.lock().unwrap();

    lobby
        .user_ids
        .iter()
        .filter(|user_id| {
            if let Some(exclude_user_id) = exclude_user_id {
                if *user_id == exclude_user_id {
                    return false;
                }
            }
            true
        })
        .filter_map(|user_id| addr_map.get_by_left(user_id))
        .filter_map(|addr| connection_map.get(addr))
        .for_each(|tx| tx.send(message.clone()).unwrap())
}

// pub fn send_message(user_id: &str, message: Message) {
//     let addr_map = USERS_TO_ADDRESSES.lock().unwrap();
//     let addr = addr_map.get_by_left(user_id);
//     let addr = if let Some(x) = addr {
//         x
//     } else {
//         return;
//     };

//     CONNECTIONS
//         .lock()
//         .unwrap()
//         .get(addr)
//         .map(|tx| tx.send(message).unwrap());
// }
