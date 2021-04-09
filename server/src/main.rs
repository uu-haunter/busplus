mod config;
mod endpoints;
mod gtfs;
mod lobby;
mod messages;
mod ws;
mod database;

use actix::Actor;
use actix_web::{App, HttpServer};

use crate::config::Config;
use crate::endpoints::ws_endpoint as ws_endpoint_route;
use crate::lobby::Lobby;
use crate::database::Connection;
use crate::database::init_connection;

const CONFIG_FILE_PATH: &str = "../config.yml";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut config_handler = Config::new();

    // If the load somehow fails the program will panic since it cannot operate
    // without the necessary data.
    if let Err(reason) = config_handler.load_config(CONFIG_FILE_PATH) {
        panic!(reason);
    }

    // Try to get the API key from the parsed cofig.
    let api_key = config_handler.get_trafiklab_value("api_key").unwrap();

    // Get Database URI from config
    let db_uri = config_handler.get_database_value("uri").unwrap();
    let conn = init_connection(db_uri);
    
    


    // Create the common/shared state.
    let lobby = Lobby::new(api_key).start();

    HttpServer::new(move || App::new().service(ws_endpoint_route).data(lobby.clone()))
        // The "0.0.0.0" means that the server accepts requests from any host (127.0.0.1, 192.168.x.x, etc..)
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
