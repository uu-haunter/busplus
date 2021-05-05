//! Everything related to managing a WebSocket connection.

use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;
use uuid::Uuid;

use crate::lobby::Lobby;
use crate::messages::{
    Connect, Disconnect, PassengerInfo, PositionUpdate, ReserveSeat, RouteRequest, UnreserveSeat,
    WsMessage,
};
use crate::protocol::client_protocol::ClientInput;
use crate::protocol::server_protocol::{ErrorType, ServerOutput};

/// How often heartbeat pings are sent.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Represents a client connected to the server via an open Websocket connection.
pub struct WebsocketClient {
    /// Address to communicate with the lobby actor.
    lobby_addr: Addr<Lobby>,

    /// Unique ID to identify each client.
    id: Uuid,

    /// Timestamp for the latest received message from the client (heartbeat).
    hb: Instant,
}

impl WebsocketClient {
    pub fn new(lobby: Addr<Lobby>) -> Self {
        WebsocketClient {
            lobby_addr: lobby,
            id: Uuid::new_v4(),
            hb: Instant::now(),
        }
    }

    /// Starts an interval which runs a function that checks if we've gotten a
    /// response from the user/any ping sent during the `CLIENT_TIMEMOUT` duration.
    pub fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // Check if the duration since we've gotten a response from the client
            // is larger than our allowed timeout time (CLIENT_TIMEOUT).
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // Send a disconnect message to the lobby before closing the connection.
                act.lobby_addr.do_send(Disconnect { self_id: act.id });

                // Stop the websocket connection to the client.
                ctx.stop();

                return;
            }

            // If the timeout threshold hasn't been passed we send a ping to the client
            // to check that they're not idle.
            ctx.ping(b"");
        });
    }
}

impl Actor for WebsocketClient {
    type Context = ws::WebsocketContext<Self>;

    // This method is called whenever a websocket connection is established with a client.
    fn started(&mut self, ctx: &mut Self::Context) {
        // Start the heartbet interval. This is really important.
        self.hb(ctx);

        // Send connect message to the lobby.
        self.lobby_addr.do_send(Connect {
            addr: ctx.address().recipient(),
            self_id: self.id,
        });
    }

    // This method is called whenever a websocket connection is broken down (disconnected).
    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        // Send disconnect message to the lobby.
        self.lobby_addr.do_send(Disconnect { self_id: self.id });

        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebsocketClient {
    // This method handles any incoming message by any client.
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // Prints what kind of message is received to the terminal if it's not a ping or pong.
        // Uncomment these lines to debug messages.
        /*
        match msg {
            Ok(ws::Message::Pong(_)) => (),
            Ok(ws::Message::Ping(_)) => (),
            _ => println!("WS: {:?}", msg),
        }
        */

        // Figure out what kind of message we've received.
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                // If the client has pinged us (the server), they are not idle so we update
                // the last heartbeat time.
                self.hb = Instant::now();

                // Send a pong message back to the user.
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                // If we've received a pong that means the client has responded to our ping,
                // so we update the last heartbeat time.
                self.hb = Instant::now();
            }
            // TODO: This should probably be changed to something else instead
            // of echoing back to the client.
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                // If we've received a close message that means the client wants
                // to disconnect so we close the session from our end and stop
                // the socket connection.
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                // Continuation frames are for fragmented WebSocket messages.
                // On receiving a continuation message we simply stop the connection
                // since we don't support this type of request.
                ctx.stop();
            }
            Ok(ws::Message::Nop) => (),
            Ok(ws::Message::Text(text)) => {
                // Try to parse the text received as a JSON representation of a ClientInput object.
                let parse_result = serde_json::from_str::<ClientInput>(&text);

                // Check if the parsing was sucessful.
                if let Ok(parsed_input) = parse_result {
                    // If it was successful, pattern match on what type of input was received.
                    match parsed_input {
                        // TODO: Handle these.
                        ClientInput::GetLineInformation(_) => (),
                        ClientInput::GetRouteInformation(inp) => {
                            self.lobby_addr.do_send(RouteRequest {
                                self_id: self.id,
                                line_number: inp.line,
                            });
                        }
                        ClientInput::GeoPositionUpdate(inp) => {
                            // Send information to the lobby that the position should be updated.
                            self.lobby_addr.do_send(PositionUpdate {
                                self_id: self.id,
                                position: inp,
                            });
                        }
                        ClientInput::GetPassengerInformation(inp) => {
                            self.lobby_addr.do_send(PassengerInfo {
                                self_id: self.id,
                                descriptor_id: inp.descriptor_id,
                            });
                        }
                        ClientInput::ReserveSeat(inp) => {
                            self.lobby_addr.do_send(ReserveSeat {
                                self_id: self.id,
                                descriptor_id: inp.descriptor_id,
                            });
                        }
                        ClientInput::UnreserveSeat => {
                            self.lobby_addr.do_send(UnreserveSeat { self_id: self.id });
                        }
                    }
                } else {
                    // If the message sent by the client is not parseable as JSON, an error message
                    // is sent back to the user.
                    ctx.text(ServerOutput::error_message(
                        ErrorType::UnknownMessage,
                        "Unsupported message".to_owned(),
                    ));
                }
            }

            // TODO: Change this panic to something else (log and disconnect?).

            // If the message sent by the client is invalid (should rarely
            // happen in theory), we panic (exit the program).
            Err(e) => panic!("{}", e),
        }
    }
}

impl Handler<WsMessage> for WebsocketClient {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
