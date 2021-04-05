mod endpoints;
mod lobby;
mod messages;
mod ws;

use actix::Actor;
use actix_web::{App, HttpServer};
use endpoints::ws_endpoint as ws_endpoint_route;
use lobby::Lobby;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let lobby = Lobby::default().start();

    HttpServer::new(move || App::new().service(ws_endpoint_route).data(lobby.clone()))
        // The "0.0.0.0" means that the server listens on any host (127.0.0.1, 192.168.x.x, etc..)
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
