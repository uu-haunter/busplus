use std::fs;
use std::path::Path;

use yaml_rust::{Yaml, YamlLoader};

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

    pub fn get_trafiklab_value(&self, key: &str) -> Option<&str> {
        // If the documents vector is empty that means we haven't loaded in any config
        // file yet, so None is returned.
        if self.documents.is_empty() {
            return None;
        }

        let document = &self.documents[0];

        document["trafiklab_api"][key].as_str()
    }
}
