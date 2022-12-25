#![allow(dead_code)]

use std::fs;
use std::path::Path;

pub struct Config {
    hermes_address: Option<String>,
    table_name: Option<String>,
    cmd_list: Vec<Cmd>,
}

impl Config {
    /// Create new config
    pub fn new() -> Self {
        Config {
            hermes_address: None,
            table_name: None,
            cmd_list: Vec::new(),
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
            .filter(|x| x.starts_with("server.grpc.address") || x.starts_with("key.") || x.starts_with("table"))
            .map(|x| String::from(x.clone()))
            .collect();

        // Process properties in config file:
        // 1. Split at '=' character
        //    a. If result lenght is less than 2, then error, go next line
        // 2. If property is "server.grpc.address" then save it, then go next line
        // 3. Split property at '_' character, format must be the followig: cmd_<table-name>_<key-name>
        //    a. If it is proper then parse the value, then save it
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

            // Save if it was Hermes address
            if attrs[0] == "server.grpc.address" {
                self.hermes_address = Some(attrs[1].to_string());
                continue;
            }

            if attrs[0] == "table" {
                self.table_name = Some(attrs[1].to_string());
                continue;
            }
            
            // Get table and key names
            let prop: Vec<&str> = attrs[0].split(".")
                .collect();

            if prop.len() < 2 {
                errors += format!("WARNING: Property is not properly specified: {}", c).as_str();
                continue;
            }

            if prop[0] != "key" {
                continue;
            }

            // Parse command
            let mut words: Vec<String> = attrs[1].split_whitespace()
                .map(|x| String::from(x))
                .collect();
            
            let cmd = words[0].clone();
            words.remove(0);
            self.cmd_list.push(Cmd::new(cmd, words, prop[1].to_string()));
        }

        if let None = self.hermes_address {
            errors += "Attribute, server.grpc.address, is not specified\n";
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
    pub fn get_cmd_list(&self) -> &Vec<Cmd> {
        return &self.cmd_list;
    }

    pub fn get_table_name(&self) -> &Option<String> {
        return &self.table_name;
    }
}

#[derive(Debug)]
pub struct Cmd {
    cmd: String,
    args: Vec<String>,
    hermes_key: String,
}

impl Cmd {
    /// Create new file
    fn new(cmd: String, args: Vec<String>, key: String) -> Self {
        Cmd {
            cmd: cmd,
            args: args,
            hermes_key: key,
        }
    }

    /// Get information about file
    pub fn get_info(&self) -> (&String, &Vec<String>, &String) {
        return (&self.cmd, &self.args, &self.hermes_key);
    }
}