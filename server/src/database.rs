//! Module for handling operations and connection to a external MongoDB database

use mongodb::bson::Document;
use mongodb::{
    error::Error,
    options::ClientOptions,
    sync::{Client, Database},
};

use crate::gtfs::transit_static::{Route, Shape, Trip};
use crate::protocol::server_protocol::RouteNode;

/// Database name for the database containing static data.
const STATIC_DATABASE: &str = "trafiklab-static-data";

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
    fn static_db(&self) -> Database {
        self.client.database(STATIC_DATABASE)
    }
}

impl DbConnection {
    /// Query the database for a "route".
    pub fn get_route(&self, query: Document) -> Result<Route, ()> {
        if let Ok(value_option) = self.static_db().collection("routes").find_one(query, None) {
            let row: Route = value_option.unwrap();

            Ok(row)
        } else {
            Err(())
        }
    }

    /// Query the database for a "trip".
    pub fn get_trip(&self, query: Document) -> Result<Trip, ()> {
        if let Ok(value_option) = self.static_db().collection("trips").find_one(query, None) {
            let row: Trip = value_option.unwrap();

            Ok(row)
        } else {
            Err(())
        }
    }

    /// Query the database for a list of "shapes".
    pub fn get_shapes(&self, query: Document) -> Result<Vec<RouteNode>, ()> {
        if let Ok(cursor) = self.static_db().collection("shapes").find(query, None) {
            // Create a vector to store all the nodes in.
            let mut nodes = Vec::new();

            for result in cursor {
                // Each "result" in the cursor iterator is a result from a mongodb
                // query since not all documents might be fetched at the same time.
                if let Ok(document) = result {
                    // Type annotate.
                    let shape: Shape = document;

                    // Store a RouteNode representation in the shapes list.
                    nodes.push(RouteNode {
                        lat: shape.shape_pt_lat,
                        lng: shape.shape_pt_lon,
                        sequence: shape.shape_pt_sequence.parse().unwrap(),
                    });
                }
            }

            // Return all the shapes.
            Ok(nodes)
        } else {
            Err(())
        }
    }
}
