//! Structs for parsing data in Trafiklab's Static API.
//!
//! The structs defined in this moduel can be used to extract data from Trafiklab's Static API
//! (https://www.trafiklab.se/api/gtfs-regional-static-data-beta).
//!
//! Here is an example of how you might go about parsing one of these datasets (in CSV format), using the
//! [csv](https://docs.rs/csv/1.1.6/csv/) crate:
//! ```edition2018
//! use csv::Reader;
//!
//! {
//!     let trips = File::open("./agency.txt").unwrap();
//!
//!     let mut rdr = Reader::from_reader(trips);
//!     let mut iter = rdr.deserialize();
//!
//!     // Iterates over every CSV record in "agency.txt"
//!     while let Some(result) = iter.next() {
//!         let record: Agency = result.unwrap();
//!
//!         println!("{:?}", record);
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};

/// Represents an agency from Trafiklab's Static API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Agency {
    pub agency_id: String,
    pub agency_name: String,
    pub agency_url: String,
    pub agency_timezone: String,
    pub agency_lang: String,
    pub agency_fare_url: Option<String>,
}

/// Represents an attribution (trip_id to organization) from Trafiklab's Static API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attributions {
    pub trip_id: String,
    pub organization_name: String,
    pub is_operator: u8,
}

/// Represents a calendar from Trafiklab's Static API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Calendar {
    pub service_id: u8,
    pub monday: u8,
    pub tuesday: u8,
    pub wednesday: u8,
    pub thursday: u8,
    pub friday: u8,
    pub saturday: u8,
    pub sunday: u8,
    pub start_date: String,
    pub end_date: String,
}

/// Represents a calendar date from Trafiklab's Static API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarDates {
    pub service_id: String,
    pub date: String,
    pub exception_type: u8,
}

/// Represents feed information from Trafiklab's Static API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedInfo {
    pub feed_id: String,
    pub feed_publisher_name: String,
    pub feed_publisher_url: String,
    pub feed_lang: String,
    pub feed_version: String,
}

/// Represents a route from Trafiklabs Static API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Route {
    pub route_id: String,
    pub agency_id: String,

    /// When both route_short_name and route_long_name have a value, the value for route_long_name should be seen
    /// as the correct name for that line. The value of route_short_name should, when route_long_name is Some, be
    /// seen as an alternative for systems that cannot show route_long_name.
    pub route_short_name: String,
    pub route_long_name: Option<String>,

    pub route_type: u16,

    /// Example: "Stadsbuss", "Regionbuss", "Sjukresebuss" etc.
    pub route_desc: Option<String>,
}

/// Represents a shape from Trafiklab's Static API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub shape_id: String,
    pub shape_pt_lat: String,
    pub shape_pt_lon: String,
    pub shape_pt_sequence: u32,
    pub shape_dist_traveled: Option<f64>,
}

/// Represents a stop time from Trafiklab's Static API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopTime {
    pub trip_id: String,
    pub arrival_time: String,
    pub departure_time: String,
    pub stop_id: String,
    pub stop_sequence: u32,
    pub stop_headsign: String,
    pub pickup_type: u8,
    pub drop_off_type: u8,
    pub shape_dist_traveled: Option<f64>,
    pub timepoint: u8,
}

/// Represents a stop from Trafiklab's Static API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stop {
    pub stop_id: String,
    pub stop_name: String,
    pub stop_lat: String,
    pub stop_lon: String,
    pub location_type: u8,
    pub parent_station: Option<String>,
    pub platform_code: Option<String>,
}

/// Represents a transfer from Trafiklab's Static API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transfer {
    pub from_stop_id: String,
    pub to_stop_id: String,
    pub transfer_type: u8,
    pub min_transfer_time: Option<String>,
    pub from_trip_id: String,
    pub to_trip_id: String,
}

/// Represents a trip from Trafiklabs Static API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trip {
    pub route_id: String,
    pub service_id: u8,
    pub trip_id: String,
    pub trip_headsign: Option<String>,
    pub direction_id: u8,
    pub shape_id: u8,
}
