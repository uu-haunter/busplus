//! Interface for receiving and storing data from Trafiklab's API:s.

use std::fs::File;
use std::io::prelude::*;
use std::str::from_utf8;

use curl::easy::Easy;
use quick_protobuf::{BytesReader, MessageRead};
use serde::{Deserialize, Serialize};
use tempdir::TempDir;
use zip::ZipArchive;

use crate::gtfs::transit_realtime::FeedMessage;

// The data the Trafiklab provides in their "GTFS Regional Realtime (Beta)" API is
// Protocol Buffer data, it needs to be decompressed and then parsed into human-readable data
// in order to be handled in a practical manner.
//
// A detailed description (in Protocol Buffer format) of the data that is retrieved from
// Trafiklab can be found here: /// https://github.com/google/transit/blob/master/gtfs-realtime/proto/gtfs-realtime.proto
//
// Read more about protocol buffers here: https://developers.google.com/protocol-buffers/docs/overview
//
// API Description URL: https://www.trafiklab.se/api/gtfs-regional-realtime-beta

/// The URL for Trafiklab's Vehicle Positions API.
const TRAFIKLAB_VEH_POS_API_URL: &str =
    "https://opendata.samtrafiken.se/gtfs-rt/ul/VehiclePositions.pb?key=";

/// The URL for Trafiklab's Static Data API.
const TRAFIKLAB_STATIC_API_URL: &str = "https://opendata.samtrafiken.se/gtfs/ul/ul.zip?key=";

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
    realtime_key: String,
    static_key: String,

    // Handle to a directory that contains static files. None means that there are no fetched
    // static files.
    static_files: Option<TempDir>,

    // Raw data received from the realtime API endpoint.
    raw_data: Vec<u8>,
}

impl TrafiklabApi {
    pub fn new(realtime_key: &str, static_key: &str) -> Self {
        TrafiklabApi {
            realtime_key: String::from(realtime_key),
            static_key: String::from(static_key),
            static_files: None,
            raw_data: Vec::new(),
        }
    }

    /// Makes a request to Trafiklab's API for static data. The files that are received from the
    /// request is stored in the OS's temporary folder (%temp% on Windows).
    pub fn fetch_static_data(&mut self) -> Result<(), ()> {
        // Byte array for the raw data received from the API.
        let mut zip_data: Vec<u8> = Vec::new();

        let mut handle = Easy::new();
        handle
            .url(&format!("{}{}", TRAFIKLAB_STATIC_API_URL, self.static_key))
            .unwrap();

        // We must use the "Accept-Encoding: gzip", since the protocol buffer data is compressed.
        handle.accept_encoding("gzip").unwrap();

        {
            let mut transfer = handle.transfer();
            transfer
                .write_function(|data| {
                    // Write the received raw data to the byte array.
                    zip_data.extend_from_slice(data);
                    Ok(data.len())
                })
                .unwrap();

            // Try to perform the request and if it fails, return.
            if let Err(_) = transfer.perform() {
                return Err(());
            }
        }

        // Create temporary directory to store the downloaded zip file in.
        let temp_dir = TempDir::new("trafiklab-static-data").unwrap();
        let zip_output_path = temp_dir.path().join("output.zip");

        {
            // Write the contents to the output file and then close the handle
            // when the scope is exited.
            let mut file = File::create(&zip_output_path).unwrap();
            file.write_all(&zip_data).unwrap();
        }

        // Open the file to get a handle for it (we cannot keep the last file handle
        // open since we get permission errors from the OS, so we reopen it).
        let file = File::open(&zip_output_path).unwrap();

        // Unzip the archive and place it's contentes into the temporary directory.
        let mut archive = ZipArchive::new(file).unwrap();
        archive.extract(temp_dir.path()).unwrap();

        // Store the handle for the temporary directory.
        self.static_files = Some(temp_dir);

        Ok(())
    }

    /// Deletes all static data (if any are downloaded).
    pub fn delete_static_data(&mut self) {
        if self.static_files.is_some() {
            // Drop the directory, essentially removing all the temporary directory
            // and all files in it.
            drop(self.static_files.as_ref());

            self.static_files = None;
        }
    }

    /// Makes a request to the Trafiklab API endpoint and stores the received data.
    /// To retrieve the data that was fetched, use `get_vehicle_positions()`.
    /// If Err(reason) is returned, reason is the error reason sent back by the Trafiklab API.
    pub fn fetch_vehicle_positions(&mut self) -> Result<(), String> {
        // Clear any previous data stored in the local buffer.
        self.raw_data.clear();

        let mut handle = Easy::new();
        handle
            .url(&format!(
                "{}{}",
                TRAFIKLAB_VEH_POS_API_URL, self.realtime_key
            ))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_vehicle_positions() {
        let handler = TrafiklabApi::new("this_doesnt_matter", "neither_does_this");
        let get_result = handler.get_vehicle_positions();

        // Since we haven't called "fetch_vehicle_positions()", we have not received
        // any data and therefore we should always get None from get_vehicle_positions().
        assert_eq!(get_result.is_none(), true);
    }

    #[test]
    fn test_bad_api_key() {
        let mut handler = TrafiklabApi::new("this_is_not_a_valid_key", "neither_does_this");
        let request_result = handler.fetch_vehicle_positions();

        // When making a request with a bad api_key an error should always be returned
        // since the API server do not accept a bad API key.
        assert_eq!(request_result.is_err(), true);
    }
}
