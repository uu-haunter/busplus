//! All possible values that the server should be able to output/send as JSON to the client.

use serde::{Deserialize, Serialize};

use crate::gtfs::transit_realtime::Position;

/// This is all possible output the server should be able to send to the
/// client. Every enumerated value in this type must have a:
///
///     #[serde(rename = "json_key_name")]
///
/// before the value declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ServerOutput {
    #[serde(rename = "vehicle-positions")]
    VehiclePositions(VehiclePositionsOutput),
}

/// Represent a list of vehicles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VehiclePositionsOutput {
    // Timestamp is POSIX TIME (seconds since 1970-01-01 00:00:00).
    pub timestamp: u64,
    pub positions: Vec<Vehicle>,
}

/// Represent a vehicle with an ID and a position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vehicle {
    pub id: String,
    pub position: Position,
}

/// Represent a line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    pub timestamp: String,
    pub line: String,
    pub vehicles: u32,
    pub stops: Vec<Stop>,
}

/// Represent a route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    pub timestamp: String,
    pub line: String,
    pub route_id: String,
    pub route: Vec<Coordinate>,
}

/// Represent a coordinate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Coordinate {
    pub lat: f32,
    pub lng: f32,
}

// Represent a vehicle with an ID and a position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stop {
    pub id: String,
    pub lines: Vec<String>,
    pub position: Position,
}
