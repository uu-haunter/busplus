use crate::gtfs::transit_realtime::FeedMessage;
use curl::easy::Easy;
use quick_protobuf::{BytesReader, MessageRead};
use serde::{Deserialize, Serialize};
use std::str::from_utf8;

/// The data the Trafiklab provides in their "GTFS Regional Realtime (Beta)" API is
/// Protocol Buffer data, it needs to be decompressed and then parsed into human-readable data
/// in order to be handled in a practical manner.
///
/// A detailed description (in Protocol Buffer format) of the data that is retrieved from
/// Trafiklab can be found here: /// https://github.com/google/transit/blob/master/gtfs-realtime/proto/gtfs-realtime.proto
///
/// Read more about protocol buffers here: https://developers.google.com/protocol-buffers/docs/overview

// API Description URL: https://www.trafiklab.se/api/gtfs-regional-realtime-beta
const TRAFIKLAB_API_URL: &str =
    "https://opendata.samtrafiken.se/gtfs-rt/ul/VehiclePositions.pb?key=";

/// Struct for representing JSON error data received from the Trafiklab API.
/// Example JSON:
///
///     {
///         errorMessage: "Key \"l33t_g4m3r\" is invalid"
///     }
///
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrafiklabError {
    pub error_message: String,
}

/// Contains necessary data in order to communicate with and receive data from Trafiklab's API.
pub struct TrafiklabApi {
    api_key: String,
    raw_data: Vec<u8>,
}

impl TrafiklabApi {
    pub fn new(api_key: &str) -> Self {
        TrafiklabApi {
            api_key: api_key.to_owned(),
            raw_data: Vec::new(),
        }
    }

    /// Makes a request to the trafiklab API endpoint and stores the received data.
    /// To retrieve the data that was fetched, use `get_vehicle_positions()`.
    /// If Err(reason) is returned, reason is the error reason sent back by the Trafiklab API.
    pub fn fetch_vehicle_positions(&mut self) -> Result<(), String> {
        let mut handle = Easy::new();
        handle
            .url(&format!("{}{}", TRAFIKLAB_API_URL, self.api_key))
            .unwrap();

        // We must use the "Accept-Encoding: gzip", since the protocol buffer data is compressed.
        handle.accept_encoding("gzip").unwrap();

        {
            let mut transfer = handle.transfer();
            transfer
                .write_function(|data| {
                    // Write the received binary data to self.raw_data.
                    self.raw_data.extend_from_slice(data);
                    Ok(data.len())
                })
                .unwrap();
            transfer.perform().unwrap();
        }

        // Check if the data received is parsable as a normal UTF-8 string.
        // If the data is parseable, that means we have not received the Protocol Buffer
        // data that was requested, but instead an error message in json.
        if let Ok(err_str) = from_utf8(&self.raw_data) {
            // Parse the str as a TrafiklabError so that we can return the reason for the error.
            if let Ok(error_message) = serde_json::from_str::<TrafiklabError>(err_str) {
                // Remove all data that was received from the API since it's not considered raw data any more.
                self.raw_data.clear();

                return Err(error_message.error_message);
            } else {
                self.raw_data.clear();
                return Err("Data not in Protocol Buffer format.".to_owned());
            }
        }

        Ok(())
    }

    /// Returns any data fetched from `fetch_vehicle_positions()`.
    /// Make sure that you've called `fetch_vehicle_positions()` before you use this functions as the output is meaningless otherwise.
    pub fn get_vehicle_positions(&self) -> Option<FeedMessage> {
        // If the length of the stored raw data is 0 that means the user hasn't
        // fetched any data yet or the fetch has previously failed.
        if self.raw_data.len() == 0 {
            return None;
        }

        let mut reader = BytesReader::from_bytes(&self.raw_data.clone());

        Some(FeedMessage::from_reader(&mut reader, &self.raw_data).unwrap())
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_get_vehicle_positions() {
        let handler = TrafiklabApi::new("this_doesnt_matter");
        let get_result = handler.get_vehicle_positions();

        // Since we haven't called "fetch_vehicle_positions()", we have not received
        // any data and therefore we should always get None from get_vehicle_positions().
        assert_eq!(get_result.is_none(), true);
    }

    #[test]
    fn test_bad_api_key() {
        let mut handler = TrafiklabApi::new("this_is_not_a_valid_key");
        let request_result = handler.fetch_vehicle_positions();

        // When making a request with a bad api_key an error should always be returned
        // since the API server do not accept a bad API key.
        assert_eq!(request_result.is_err(), true);
    }
}
