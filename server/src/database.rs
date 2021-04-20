//! Module for handling operations and connection to a external MongoDB database
use mongodb::{error::Error, options::ClientOptions, sync::Client};

/// Constant of the database name that should be used
const DATABASE_NAME: &str = "haunterDB";

/// Our abstraction for the db, we can use method syntax for operation ex: conn.updateGeoPosition(id, value)
pub struct DbConnection {
    client: Client,
}

/// Inititalise a connection with uri_str
pub fn init_db_connection(uri_str: &str) -> Result<DbConnection, Error> {
    let client_options = ClientOptions::parse(uri_str)?;
    let result_client = Client::with_options(client_options)?;

    Ok(DbConnection {
        client: result_client,
    })
}

impl DbConnection {
    // TODO: Code for operations upon the db
}
