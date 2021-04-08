use serde::{Deserialize, Serialize};

use crate::gtfs::transit_realtime::Position;

/// This files declares all possible values that the client should be able
/// to send as JSON to the server.

/// This is all possible inputs the server should be able to receive from a
/// client. Every enumerated value in this type must have a:
///
///     #[serde(rename = "json_key_name")]
///
/// before the value declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientInput {
    #[serde(rename = "geo-position-update")]
    GeoPositionUpdate(GeoPositionInput),
}

// Position data from the client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeoPositionInput {
    // The radius is the maximum distance (in metres) from the clients position
    // on their map.
    pub radius: i32,
    pub position: Position,
}
