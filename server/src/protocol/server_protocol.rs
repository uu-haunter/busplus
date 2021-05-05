//! All possible values that the server should be able to output/send as JSON to the client.

use serde::{Deserialize, Serialize};

use crate::gtfs::transit_realtime::Position;

/// Defines possible errors that might occur on the server side.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorType {
    ServerError,
    UnknownMessage,
    BadData,
    Position,
    LineInfo,
    RouteInfo,
    Reserve,
    Unreserve,
}

/// This is all possible output the server should be able to send to the
/// client. Every enumerated value in this type must have a:
///
///     #[serde(rename = "json_key_name")]
///
/// before the value declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ServerOutput {
    #[serde(rename = "error")]
    Error(ErrorOutput),

    #[serde(rename = "vehicle-positions")]
    VehiclePositions(VehiclePositionsOutput),

    #[serde(rename = "passenger-info")]
    PassengerInformation(PassengerInformationOutput),

    #[serde(rename = "route-info")]
    RouteInformation(RouteInformationOutput),
}

impl ServerOutput {
    pub fn error_message(error_type: ErrorType, error_message: String) -> String {
        serde_json::to_string(&ServerOutput::Error(ErrorOutput {
            error_type,
            error_message,
        }))
        .unwrap()
    }
}

/// Represent an error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorOutput {
    pub error_type: ErrorType,
    pub error_message: String,
}

/// Represent a list of vehicles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VehiclePositionsOutput {
    // Timestamp is POSIX TIME (seconds since 1970-01-01 00:00:00).
    pub timestamp: u64,
    pub vehicles: Vec<Vehicle>,
}

/// Represent a vehicle with an ID and a position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vehicle {
    pub descriptor_id: String,
    pub line: Option<String>,
    pub trip_id: Option<String>,
    pub position: Position,
}

/// Represents passenger information for a bus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PassengerInformationOutput {
    pub capacity: i32,
    pub passengers: i32,
}

/// Represent a list of lines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteInformationOutput {
    pub timestamp: u64,
    pub line: String,
    pub route_id: String,
    pub route: Vec<RouteNode>,
}

/// Represent a route node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteNode {
    pub lat: String,
    pub lng: String,
    pub sequence: i32,
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
