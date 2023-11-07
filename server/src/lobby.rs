lazy_static! {
    static ref LOBBIES: Mutex<HashMap<String, UnboundedSender<Message>>> =
        Mutex::new(HashMap::new());
}
