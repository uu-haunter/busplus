use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;
use uuid::Uuid;

use crate::lobby::Lobby;
use crate::messages::{Connect, Disconnect, WsMessage};

// How often heartbeat pings are sent.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

// How long before lack of client response causes a timeout.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

// Represents a client connected to the server via an open Websocket connection.
pub struct WebsocketClient {
    lobby_addr: Addr<Lobby>,
    id: Uuid,
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

    pub fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // TODO: Remove this in the future. Only here to show that the
                // event occurred.
                println!("Websocket client heartbeat failed, disconnecting.");

                // Send a disconnect message to the lobby.
                act.lobby_addr.do_send(Disconnect { self_id: act.id });

                ctx.stop();

                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for WebsocketClient {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        // Send connect message to the lobby.
        self.lobby_addr.do_send(Connect {
            addr: ctx.address().recipient(),
            self_id: self.id,
        });
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        // Send disconnect message to the lobby.
        self.lobby_addr.do_send(Disconnect { self_id: self.id });

        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebsocketClient {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Pong(_)) => (),
            Ok(ws::Message::Ping(_)) => (),
            _ => println!("WS: {:?}", msg),
        }

        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            // TODO: This should probably be changed to something else instead
            // of echoing back to the client.
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
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
                // TODO: Here we should parse incoming messages (as JSON?).
                // It could also be useful to send messages to the lobby here in
                // order to echo out to all connected clients of some event at a
                // later point in time

                // For now we echo back to the user what they've sent.
                ctx.text(text);
            }

            // TODO: Change this panic to something else (log and disconnect?).

            // If the message sent by the client is invalid (should rarely
            // happen in theory), we panic (exit the program).
            Err(e) => panic!(e),
        }
    }
}

impl Handler<WsMessage> for WebsocketClient {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
