//! Module for handling operations and connection to a external MongoDB database
use mongodb::{error::Error, options::ClientOptions, Client};

///Constant of the database name that should be used
const DATABASE_NAME: &str = "haunterDB";

///Our abstraction for the db, we can use method syntax for operation ex: conn.updateGeoPosition(id, value)
pub struct Connection {
    client: Client,
}

///Inititalise a connection with uri_str
pub async fn init_connection(uri_str: &str) -> Result<Connection, mongodb::error::Error> {
    let client_options = ClientOptions::parse(uri_str).await?;
    let result_client = Client::with_options(client_options)?;
    let result = Connection {
        client: result_client,
    };
    Ok(result)
}

impl Connection {
    //TODO: Code for operations upon the db
}
