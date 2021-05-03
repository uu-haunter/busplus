//! Module for handling operations and connection to a external MongoDB database

use mongodb::bson::{from_bson, Bson, Document};
use mongodb::{error::Result, options::ClientOptions, Client, Database};
use tokio::stream::StreamExt;

use crate::gtfs::transit_static::{Route, Shape, Trip};
use crate::protocol::server_protocol::RouteNode;

/// Database name for the database containing static data.
const STATIC_DATABASE: &str = "trafiklab-static-data";

/// Our abstraction for the db, we can use method syntax for operation ex: conn.updateGeoPosition(id, value)
#[derive(Clone)]
pub struct DbConnection {
    client: Client,
}

/// Inititalise a connection with uri_str
pub async fn init_db_connection(uri_str: &str) -> Result<DbConnection> {
    let client_options = ClientOptions::parse(uri_str).await?;
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
    pub async fn get_route(&self, query: Document) -> Option<Route> {
        match self
            .static_db()
            .collection("routes")
            .find_one(query, None)
            .await
        {
            Ok(value_option) => match value_option {
                Some(doc) => Some(from_bson(Bson::Document(doc)).unwrap()),
                None => None,
            },
            Err(_) => None,
        }
    }

    /// Query the database for a "trip".
    pub async fn get_trip(&self, query: Document) -> Option<Trip> {
        match self
            .static_db()
            .collection("trips")
            .find_one(query, None)
            .await
        {
            Ok(value_option) => match value_option {
                Some(doc) => Some(from_bson(Bson::Document(doc)).unwrap()),
                None => None,
            },
            Err(_) => None,
        }
    }

    /// Query the database for a list of "shapes".
    pub async fn get_shapes(&self, query: Document) -> Option<Vec<RouteNode>> {
        match self
            .static_db()
            .collection("shapes")
            .find(query, None)
            .await
        {
            Ok(mut cursor) => {
                // Create a vector to store all the nodes in.
                let mut nodes = Vec::new();

                while let Some(result) = cursor.next().await {
                    // Each "result" in the cursor iterator is a result from a mongodb
                    // query since not all documents might be fetched at the same time.
                    if let Ok(document) = result {
                        // Type annotate.
                        let shape: Shape = from_bson(Bson::Document(document)).unwrap();

                        // Store a RouteNode representation in the shapes list.
                        nodes.push(RouteNode {
                            lat: shape.shape_pt_lat,
                            lng: shape.shape_pt_lon,
                            sequence: shape.shape_pt_sequence.parse().unwrap(),
                        });
                    }
                }

                // Return all the shapes.
                Some(nodes)
            }
            Err(_) => None,
        }
    }
}
