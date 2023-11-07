use std::{collections::HashMap, net::SocketAddr, sync::Mutex};

use futures_util::{
    future::{ok, select},
    pin_mut, StreamExt, TryStreamExt,
};
use tokio::{
    net::TcpStream,
    sync::mpsc::{unbounded_channel, UnboundedSender},
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_tungstenite::tungstenite::Message;

lazy_static! {
    /*
     * Holds a map of users -> broadcastable channels.
     * TODO: I think we want to use user_id instead of the address.
     */
    static ref CONNECTIONS: Mutex<HashMap<String, UnboundedSender<Message>>> = Mutex::new(HashMap::new());
}

pub async fn handle_connection(stream: TcpStream, addr: SocketAddr) {
    println!("New connection: {:?}", addr);

    let ws = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during websocket handshake occurred.");

    let (tx, rx) = unbounded_channel();
    CONNECTIONS.lock().unwrap().insert(addr.to_string(), tx);

    let (outgoing, incoming) = ws.split();

    // For now, broadcast the message to everyone but ourselves.
    let incoming_stream = incoming.try_for_each(|msg| {
        println!("Received message from {}", addr);

        let peers = CONNECTIONS.lock().unwrap();
        let recipients = peers
            .iter()
            .filter(|(peer_addr, _)| peer_addr != &&addr.to_string())
            .map(|(_, ws_sink)| ws_sink);

        recipients.for_each(|ws_sink| {
            ws_sink.send(msg.clone()).unwrap();
        });

        ok(())
    });

    let rx_stream = UnboundedReceiverStream::new(rx);
    let receive_from_others = rx_stream.map(Ok).forward(outgoing);

    pin_mut!(incoming_stream, receive_from_others);
    select(incoming_stream, receive_from_others).await;

    println!("{} disconnected", addr);
    CONNECTIONS.lock().unwrap().remove(&addr.to_string());
}

pub fn send_message(user_id: &str, message: String) {
    CONNECTIONS
        .lock()
        .unwrap()
        .get(user_id)
        .map(|tx| tx.send(Message::Text(message)).unwrap());
}
