use crate::lobby::Lobby;
use crate::ws::WebsocketClient;
use actix::Addr;
use actix_web::{get, web::Data, web::Payload, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

#[get("/ws")]
pub async fn ws_endpoint(
    req: HttpRequest,
    stream: Payload,
    srv: Data<Addr<Lobby>>,
) -> Result<HttpResponse, Error> {
    let ws = WebsocketClient::new(srv.get_ref().clone());
    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}
