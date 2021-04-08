//! Information about individual clients that the lobby needs to keep track of.

use uuid::Uuid;

use crate::gtfs::transit_realtime::Position;
use crate::lobby::Socket;

/// Used to represent the positon of a client on their map. This is used to determined
/// what data should be sent to the client depending on what can be seen on their
/// map.
#[derive(Debug)]
pub struct ClientPosition {
    pub radius: i32,
    pub position: Position,
}

/// State for a WebsocketClient. Holds information specific to each connection.
#[derive(Debug)]
pub struct ClientData {
    // Unique id.
    pub id: Uuid,

    // An address to communicate with the client actor.
    pub addr: Socket,

    // Where the client is currently positioned on their map. Used to send
    // relevant data to each individual client.
    pub position: Option<ClientPosition>,
}

impl ClientData {
    /// Constructs a new client with no position.
    pub fn new(id: Uuid, addr: Socket) -> Self {
        ClientData {
            id,
            addr,
            position: None,
        }
    }

    pub fn update_position(&mut self, position: ClientPosition) {
        self.position = Some(position);
    }
}
