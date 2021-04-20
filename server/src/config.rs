//! Utility for reading values from a config file.

use std::fs;
use std::path::Path;

use yaml_rust::{Yaml, YamlLoader};

/// The file path to the config file.
pub const CONFIG_FILE_PATH: &str = "../config.yml";

/// The YAML key where all trafiklab data is stored.
/// Example:
///
///     trafiklab_api: <-- this is the key
///         api_key: a12b34c567d89
///

const TRAFIKLAB_YAML_KEY: &str = "trafiklab_api";
const DATABASE_YAML_KEY: &str = "database";

/// Stores the parsed contents of a YAML config file.
pub struct Config {
    documents: Vec<Yaml>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            documents: Vec::new(),
        }
    }

    /// Loads the config from a file path and tries to parse it from YAML.
    pub fn load_config(&mut self, config_path: &str) -> Result<(), String> {
        // Read the contents of the config file into a string.
        let config_file_contents = fs::read_to_string(Path::new(config_path));

        if config_file_contents.is_err() {
            return Err(format!(
                "Could not load config file from {:?}.",
                config_path
            ));
        }

        // Parse the contents of the config file to YAML.
        let config = YamlLoader::load_from_str(&config_file_contents.unwrap());

        if config.is_err() {
            return Err("Could not parse contents of the config file into YAML.".to_owned());
        }

        // Store the parsed YAML contents.
        self.documents = config.unwrap();

        Ok(())
    }
    /// Gets a key value from the config section in the config file.
    pub fn get_config_value(&self, field: &str, key: &str) -> Option<&str> {
        // If the documents vector is empty that means we haven't loaded in any config
        // file yet, so None is returned.
        if self.documents.is_empty() {
            return None;
        }

        let document = &self.documents[0];

        document[field][key].as_str()
    }

    pub fn get_trafiklab_value(&self, key: &str) -> Option<&str> {
        self.get_config_value(TRAFIKLAB_YAML_KEY, key)
    }

    pub fn get_database_value(&self, key: &str) -> Option<&str> {
        self.get_config_value(DATABASE_YAML_KEY, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempdir::TempDir;

    const TEST_DIRECTORY_NAME: &str = "config_test_dir";
    const TEST_FILE_NAME: &str = "test_config.yml";
    const TEST_API_KEY: &str = "api_key";
    const TEST_API_KEY_VALUE: &str = "a12b34c567d89";

    const TEST_DATABASE_KEY: &str = "test_uri";
    const TEST_DATABASE_KEY_VALUE: &str = "testconnectionstring";

    #[test]
    fn test_config() -> std::io::Result<()> {
        let yaml_content: String = format!(
            "
{}:
  {}: {}
{}:
  {}: {}
",
            TRAFIKLAB_YAML_KEY,
            TEST_API_KEY,
            TEST_API_KEY_VALUE,
            DATABASE_YAML_KEY,
            TEST_DATABASE_KEY,
            TEST_DATABASE_KEY_VALUE
        );

        // Create a temporary directory to create the config file in.
        let dir = TempDir::new(TEST_DIRECTORY_NAME)?;
        let file_path = dir.path().join(TEST_FILE_NAME);
        eprintln!("FILE PATH: {:?}", file_path);

        // Create the file containing the yaml content.
        let mut test_file = File::create(&file_path)?;
        test_file.write_all(yaml_content.as_bytes())?;

        let mut config_handler = Config::new();

        // Make sure that the config is properly loaded in.
        assert_eq!(
            config_handler
                .load_config(file_path.to_str().unwrap())
                .is_ok(),
            true
        );

        // Make sure that some bad key returns None
        assert_eq!(
            config_handler.get_trafiklab_value("bad_key").is_none(),
            true
        );
        assert_eq!(config_handler.get_database_value("bad_key").is_none(), true);

        let get_key_result_database = config_handler.get_database_value(TEST_DATABASE_KEY);
        let get_key_result_trafik = config_handler.get_trafiklab_value(TEST_API_KEY);

        // Make sure that a correct key is returned as Some and the correct value.
        assert_eq!(get_key_result_trafik.is_some(), true);
        assert_eq!(get_key_result_trafik.unwrap(), TEST_API_KEY_VALUE);

        assert_eq!(get_key_result_database.is_some(), true);
        assert_eq!(get_key_result_database.unwrap(), TEST_DATABASE_KEY_VALUE);

        // Close the temporary directory.
        dir.close()?;

        Ok(())
    }
}
