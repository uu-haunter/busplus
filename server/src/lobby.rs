use std::collections::HashMap;

use actix::prelude::{Actor, Context, Handler, Recipient};
use uuid::Uuid;

use crate::gtfs::trafiklab::TrafiklabApi;
use crate::messages::{Connect, Disconnect, WsMessage};

// Type alias, which is essentially an address to an actor which you can
// send messages to.
type Socket = Recipient<WsMessage>;

// The lobby keeps track of a common/shared state between all clients.
pub struct Lobby {
    // Maps client IDs to a Socket.
    sessions: HashMap<Uuid, Socket>,

    // Handle to communicate with Trafiklab's API.
    trafiklab: TrafiklabApi,
}

impl Lobby {
    pub fn new(api_key: &str) -> Self {
        Lobby {
            sessions: HashMap::new(),
            trafiklab: TrafiklabApi::new(api_key),
        }
    }
}

impl Lobby {
    // Sends a message to a specific client.
    fn send_message(&self, message: &str, id_to: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient.do_send(WsMessage(message.to_owned()));
        } else {
            println!("Attempting to send message but couldn't find client id.");
        }
    }

    // Sends a message to every connected client stored in self.sessions.
    fn send_to_everyone(&self, message: &str) {
        self.sessions
            .keys()
            .for_each(|client_id| self.send_message(message, client_id));
    }

    // Sends a message to every connected client stored in self.sessions.
    fn send_to_everyone_except_self(&self, message: &str, self_id: &Uuid) {
        self.sessions
            .keys()
            .filter(|client_id| *client_id.to_owned() != *self_id)
            .for_each(|client_id| self.send_message(message, client_id));
    }
}

// This is a blanket implementation to make sure that the Lobby type is considered an Actor.
impl Actor for Lobby {
    type Context = Context<Self>;
}

impl Handler<Connect> for Lobby {
    type Result = ();

    // This method is called whenever the Lobby receives a "Connect" message.
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        // Store the address of the client in the sessions hashmap.
        self.sessions.insert(msg.self_id, msg.addr);

        // TODO: Remove this println. Only here to show that events occur.
        println!("Client with id '{}' connected.", msg.self_id);
    }
}

impl Handler<Disconnect> for Lobby {
    type Result = ();

    // This method is called whenever the Lobby receives a "Disconnect" message.
    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        // Try and remove the client from the sessions hashmap.
        if self.sessions.remove(&msg.self_id).is_some() {
            // TODO: Remove this println. Only here to show that events occur.
            println!("Client with id '{}' disconnected.", msg.self_id);
        }
    }
}
