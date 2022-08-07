use std::env;
use std::process::exit;
use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::{Arc, RwLock};

mod network;
mod services;

use services::process::Pool;
use services::data::Database;
use services::parser;

fn main() 
{
    // Read the arguments and parse it onto a structure
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Config path must be specified as parameter!");
        return;
    }

    // Parse argument from config file
    let mut config: HashMap<String, String> = match onlyati_config::read_config(args[1].as_str()) {
        Ok(conf) => conf,
        Err(e) => {
            println!("Error during config reading: {}", e);
            exit(1);
        },
    };

    if let None = config.get("threads") {
        if let Some(n) = number_of_cores() {
            config.insert(String::from("threads"), n);
        }
    }

    println!("Settings:");
    for setting in &config {
        println!("- {}: {}", setting.0, setting.1);
    }

    // If some necesarry item is missign return with error
    if let Err(error) = validate_settings(&config) {
        println!("{error}");
        exit(2);
    }

    // Initialize database
    let mut db = Database::new();
    db.create_table(String::from("Default")).unwrap();
    let db = Arc::new(RwLock::new(db));

    match config.get("init_data") {
        Some(value) => {
            println!("Read initial data from {} file", value);
            let path = std::path::Path::new(value);
            if path.exists() {
                let content = match std::fs::read_to_string(&path) {
                    Ok(content) => content,
                    Err(e) => {
                        println!("Failed to read init data file: {:?}", e);
                        String::new()
                    },
                };

                for line in content.lines() {
                    if line.is_empty() {
                        continue;
                    }

                    if let Err(e) = parser::parse_db_command(line, db.clone()) {
                        println!("Error in \"{}\" statment: {}", line, e);
                    }
                }
            }
        },
        None => println!("No init data file is specified"),
    }

    // Execute background worker threads for TCP stream
    let core_num = config.get("threads").unwrap().parse::<usize>().unwrap();
    let stream_workers = match Pool::new(core_num) {
        Ok(pool) => pool,
        Err(e) => {
            println!("ERROR during pool creation: {}", e);
            exit(3);
        }
    };

    // Bind TCPIP address
    let addr = config.get("address").unwrap();
    println!("Bind socket to '{}' address", addr);
    let listener = match TcpListener::bind(addr) {
        Ok(listener) => listener,
        Err(e) => {
            println!("ERROR: {}", e);
            exit(4);
        }
    };

    println!("Listeting on {} address...", addr);
    for stream in listener.incoming() {
        let db = db.clone();
        if let Ok(stream) = stream {
            stream_workers.execute(move || {
                network::handle_connection(stream, db);
            }).unwrap();
        }
    }
}

fn validate_settings (settings: &HashMap<String, String>) -> Result<(), String> {
    let mut errors = String::new();

    if let None = settings.get("address") {
        errors += "ERROR in config: Field 'address' is missing\n";
    }

    if let None = settings.get("threads") {
        errors += "ERROR in config: Field 'threads' is missing or couldn't fetch from /proc/cpuinfo\n";
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    return Ok(());
}

fn number_of_cores() -> Option<String> {
    match std::fs::read_to_string("/proc/cpuinfo") {
        Ok(text) => {
            for line in text.lines() {
                if line.contains(":") {
                    let temp: Vec<&str> = line.split(":").collect();
                    if temp[0].trim() == "cpu cores" {
                        match temp[1].trim().parse::<usize>() {
                            Ok(_) => return Some(String::from(temp[1].trim())),
                            Err(_) => return None,
                        }
                    }
                }
            }
        }
        Err(_) => return None,
    }

    return None;
}
