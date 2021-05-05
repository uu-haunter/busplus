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

    /// Keeps track of the last descriptor id that the client has made any request for/on.
    /// This is used to be able to send the client updates to the descriptor id when they happen.
    pub last_descriptor_request: Option<String>,
}

impl ClientData {
    /// Constructs a new client with no position and no reserved seat.
    pub fn new(id: Uuid, addr: Socket) -> Self {
        ClientData {
            id,
            addr,
            position: None,
            reserved_seat: None,
            last_descriptor_request: None,
        }
    }

    /// Updates the clients position.
    pub fn update_position(&mut self, position: GeoPosition) {
        self.position = Some(position);
    }

    /// Updates the last descriptor.
    pub fn update_last_descriptor(&mut self, descriptor_id: String) {
        self.last_descriptor_request = Some(descriptor_id);
    }
}
