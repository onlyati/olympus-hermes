#![allow(dead_code)]

use std::fs;
use std::path::Path;

pub struct Config {
    hermes_address: Option<String>,
    file_list: Vec<File>,
}

impl Config {
    /// Create new config
    pub fn new() -> Self {
        Config {
            hermes_address: None,
            file_list: Vec::new(),
        }
    }

    /// This function reads specified file and try to parse it. It filter for Hermes address and file lines.
    /// If the format is not proper then line is not processed, but skipped and write warning about them.
    pub fn parse_config(&mut self, paths: Vec<&Path>) -> Result<(), String> {
        let mut errors = String::new();
        let mut whole_config = String::new();

        // Frist we check that config files are exist and read them if we can
        for p in paths.iter() {
            if !p.exists() {
                errors += format!("ERROR: Specifiec config does not exist: {}", p.display()).as_str();
                continue;
            }

            match fs::read_to_string(p) {
                Ok(conf) => whole_config += format!("{}\n", conf).as_str(),
                Err(e) => errors += format!("ERROR: Config file cannot be read: {} {}", p.display(), e).as_str(),
            }
        }

        // Parse the read string into vector per line, then filter the information contained lines
        let config: Vec<&str> = whole_config.split("\n").collect();
        let config: Vec<String> = config.iter()
            .filter(|x| x.starts_with("hermes_address") || x.starts_with("file_"))
            .map(|x| String::from(x.clone()))
            .collect();

        // Process properties in config file:
        // 1. Split at '=' character
        //    a. If result lenght is less than 2, then error, go next line
        // 2. If property is "hermes_address" then save it, then go next line
        // 3. Split property at '_' character, format must be the followig: file_<table-name>_<key-name>
        //    a. If it is proper, then save it
        //    b. If does not fit for format, then error
        // 4. Go next line
        for c in config {
            let attrs: Vec<&str> = c.split("=")
                .map(|x| x.trim())
                .collect();

            if attrs.len() < 2 {
                errors += format!("WARNING: Line is not properly specified: {}\n", c).as_str();
                continue;
            }

            if attrs[0] == "hermes_address" {
                self.hermes_address = Some(attrs[1].to_string());
                continue;
            }
            
            let prop: Vec<&str> = attrs[0].split("_")
                .collect();

            if prop.len() < 3 {
                errors += format!("WARNING: Property is not properly specified: {}", c).as_str();
                continue;
            }

            self.file_list.push(File::new(attrs[1], prop[1], prop[2]));
        }

        if let None = self.hermes_address {
            errors += "Attribute, hermes_address, is not specified\n";
        }

        if errors.is_empty() {
            return Ok(());
        }

        return Err(errors);
    }

    /// Get value of Hermes address field
    pub fn get_hermes_address(&self) -> &Option<String> {
        return &self.hermes_address;
    }

    /// Get file list of config
    pub fn get_file_list(&self) -> &Vec<File> {
        return &self.file_list;
    }
}

#[derive(Debug)]
pub struct File {
    path: String,
    hermes_table: String,
    hermes_key: String,
}

impl File {
    /// Create new file
    fn new(path: &str, table: &str, key: &str) -> Self {
        File {
            path: String::from(path),
            hermes_key: String::from(key),
            hermes_table: String::from(table),
        }
    }

    /// Get information about file
    pub fn get_info(&self) -> (&String, &String, &String) {
        return (&self.path, &self.hermes_table, &self.hermes_key);
    }
}