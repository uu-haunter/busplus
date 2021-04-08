use actix::prelude::{Message, Recipient};
use uuid::Uuid;

use crate::gtfs::transit_realtime::Position;

// Messages in this file is used for internal communication between different
// actors (Lobby and WebsocketClient for example).

// WebsocketClient responds to this to pipe it through to the actual client.
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

// WebsocketClient sends this to connect to the lobby.
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsMessage>,
    pub self_id: Uuid,
}

// WebsocketClient sends this to disconnect from the lobby.
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub self_id: Uuid,
}

// WebsocketClient sends this to update their position (on the map in the client) in the lobby.
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct PositionUpdate {
    pub self_id: Uuid,
    pub radius: i32,
    pub position: Position,
}
