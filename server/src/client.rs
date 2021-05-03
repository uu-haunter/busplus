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

    /// None if the client has not reserved a seat on a bus, Some with a descriptor_id
    /// if the client has reserved a seat.
    pub reserved_seat: Option<String>,
}

impl ClientData {
    /// Constructs a new client with no position and no reserved seat.
    pub fn new(id: Uuid, addr: Socket) -> Self {
        ClientData {
            id,
            addr,
            position: None,
            reserved_seat: None,
        }
    }

    /// Updates the clients position.
    pub fn update_position(&mut self, position: GeoPosition) {
        self.position = Some(position);
    }
}
