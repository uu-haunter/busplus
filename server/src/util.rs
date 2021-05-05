//! Utilities. Functions that do not belong anywhere else.

use geoutils::Location;

use crate::protocol::client_protocol::GeoPosition;
use crate::protocol::server_protocol::Vehicle;

/// Returns a bool representing whether a vehicle is close enough to be sent out.
/// To be used with a filter function.
pub fn filter_vehicle_position(client_geo: &GeoPosition, vhc: &Vehicle) -> bool {
    let v_pos = Location::new(vhc.position.latitude, vhc.position.longitude);

    let client_pos = Location::new(
        client_geo.position.coordinates[0],
        client_geo.position.coordinates[1],
    );

    let distance = v_pos.distance_to(&client_pos).unwrap();

    distance.meters() < client_geo.max_distance.into()
}
