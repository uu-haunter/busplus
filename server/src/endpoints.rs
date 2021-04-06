use crate::lobby::Lobby;
use crate::ws::WebsocketClient;
use actix::Addr;
use actix_web::{get, web::Data, web::Payload, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

// This file contains all HTTP endpoints that are exposed by the webserver.

// This endpoint function is called whenever a request is sent to the "/ws" route.
#[get("/ws")]
pub async fn ws_endpoint(
    req: HttpRequest,
    stream: Payload,
    srv: Data<Addr<Lobby>>,
) -> Result<HttpResponse, Error> {
    // Create a new WebsocketClient with an address to the lobby.
    let ws = WebsocketClient::new(srv.get_ref().clone());

    // Start the websocket connection and return the result.
    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}
