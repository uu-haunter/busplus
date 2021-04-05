use actix::prelude::{Message, Recipient};
use uuid::Uuid;

// Messages in this file is used for internal communication between different
// actors (Lobby and WebsocketClient for example).

// WebsocketClient responds to this to pipe it though to the actual client.
#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

// WebsocketClient sends this to connect to the lobby.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsMessage>,
    pub self_id: Uuid,
}

// WebsocketClient sends this to disconnect from the lobby.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub self_id: Uuid,
}
