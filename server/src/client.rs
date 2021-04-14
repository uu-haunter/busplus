//! Information about individual clients connected to the lobby.

use uuid::Uuid;

use crate::lobby::Socket;
use crate::protocol::client_protocol::GeoPosition;

/// State for a WebsocketClient. Holds information specific to each connection.
#[derive(Debug)]
pub struct ClientData {
    /// Unique id.
    pub id: Uuid,

    /// An address to communicate with the client actor.
    pub addr: Socket,

    /// Where the client is currently positioned on their map. Used to send
    /// relevant data to each individual client.
    pub position: Option<GeoPosition>,
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

    /// Updates the clients position.
    pub fn update_position(&mut self, position: GeoPosition) {
        self.position = Some(position);
    }
}
