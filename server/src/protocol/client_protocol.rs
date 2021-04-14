//! All possible values that the client should be able to send as JSON to the server.

use serde::{Deserialize, Serialize};

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
    GeoPositionUpdate(GeoPosition),
}

/// Position data from the client. Contains maximum distance and a position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeoPosition {
    // The maximum distance (in metres) from the clients position on their map that information
    // should be gathered from.
    pub max_distance: i32,

    // The client's position.
    pub position: GeoPositionPoint,
}

/// GeoJSON "Point" representation, see https://geojson.org/ for more details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeoPositionPoint {
    // Cannot name the struct field "type" since that is a reserved keyword in rust.
    #[serde(rename = "type")]
    pub position_type: String,

    // The vector usuaully only have two values [longitude, latitude].
    pub coordinates: Vec<f32>,
}
