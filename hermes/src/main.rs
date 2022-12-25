use std::env;
use std::process::exit;
use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::{RwLock};

mod network;
mod services;

use services::process::Pool;
use services::data::Database;
use services::parser;
use services::grpc;
use services::agent;
use services::agent::Agent;

static VERSION: &str = "v.0.1.3";
static DB: RwLock<Option<Database>> = RwLock::new(None);
static AGENTS: RwLock<Option<HashMap<String, Agent>>> = RwLock::new(None);


fn main() 
{
    // Display version number to make sure which version is starting
    println!("Starting {} version is in progress...", VERSION);

    // Read the arguments and parse it onto a structure
    let config = match read_config() {
        Ok(conf) => conf,
        Err(e) => {
            println!("ERROR: {}", e);
            exit(1);
        }
    };

    // Initialize database
    match initialize_db(&config) {
        Ok(_) => (),
        Err(e) => {
            println!("ERROR: {}", e);
            exit(1);
        }
    }

    // Start gRPC server
    match config.get("host.grpc.address") {
        Some(addr) => {
            let addr = addr.clone();
            let core_num = config.get("options.threads").unwrap().parse::<usize>().unwrap();
            std::thread::spawn(move || {
                println!("Starting gRPC server...");
                let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(core_num)
                .build()
                .unwrap();
                rt.block_on(async move {
                    grpc::start_server(&addr).await;
                });
            });
        },
        None => println!("Address for gRPC is not specified, not started"),
    }

    // Start agents
    println!("Checking for agents...");
    match config.get("agent.enable") {
        Some(v) => {
            if v == "yes" {
                let agent_conf: HashMap<String, String> = config.iter()
                    .filter(|x| x.0.starts_with("agent."))
                    .map(|x| (x.0.clone(), x.1.clone()))
                    .collect();
                if agent_conf.len() == 0 {
                    println!("No agent related config found!");
                }
                else {
                    setup_agents(agent_conf, config.get("agent.delay"));
                }
            }
        },
        None => println!("Agent function is disabled"),
    }

    // Execute background worker threads for TCP stream
    let core_num = config.get("options.threads").unwrap().parse::<usize>().unwrap();
    let stream_workers = match Pool::new(core_num) {
        Ok(pool) => pool,
        Err(e) => {
            println!("ERROR during pool creation: {}", e);
            exit(3);
        }
    };

    // Bind TCPIP address
    let addr = config.get("host.classic.address").unwrap();
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
        if let Ok(stream) = stream {
            stream_workers.execute(move || {
                network::handle_connection(stream);
            }).unwrap();
        }
    }
}

/// ## Setup agents
/// 
/// This function read the agent related configuration and start the pleaseured agents.
fn setup_agents(config: HashMap<String, String>, init_sleep: Option<&String>) {
    let mut ids: HashMap<String, u64> = HashMap::new();

    let base_dir = match config.get("agent.dir.bin") {
        Some(p) => p,
        None => {
            eprintln!("Did not found agent_bin_dir in config file, no agent will be started");
            return;
        },
    };

    let conf_dir = match config.get("agent.dir.conf") {
        Some(p) => p,
        None => {
            eprintln!("Did not found agent_conf_dir in config file, no agent will be started");
            return;
        },
    };

    let init_sleep = match init_sleep {
        Some(int) => int,
        None => {
            eprintln!("No initial delay specified for agents, no agent will be started");
            return;
        }
    };

    let init_sleep: u64 = match init_sleep.parse() {
        Ok(r) => r,
        Err(_) => {
            eprintln!("Parse failed for agent_initial_delay, must be a positive number");
            return;
        }
    };

    for item in &config {
        let props: Vec<&str> = item.0.split(".").collect();

        if props.len() == 3 {
            if props[0] == "agent" && props[2] == "interval" {
                let id = String::from(props[1]);
                let interval = match item.1.parse::<u64>() {
                    Ok(r) => r,
                    Err(_) => {
                        eprintln!("Failed to parse interval for agent: {} {}", item.0, item.1);
                        continue;
                    }
                };
                ids.insert(id, interval);
            }
        }
    }

    for item in &ids {
        let exe_path = format!("{}/agent_{}.d/agent_{}", base_dir, item.0, item.0);
        let conf_path = vec![
            format!("{}/common.conf", conf_dir),
            format!("{}/agent_{}.d/agent_{}.conf", conf_dir, item.0, item.0)
        ];
        let log_path = format!("{}/agent_{}.d/agent_{}.log", conf_dir, item.0, item.0);

        let agent = Agent::new(item.0.clone(), item.1.clone(), exe_path, log_path, conf_path);

        {
            let mut db = DB.write().unwrap();
            if let Some(db) = &mut *db {
                let table_name = format!("{}{}Agent", &item.0[0..1].to_uppercase(), &item.0[1..]);
                println!("Table {} is creating for agent {}", table_name, item.0);
                let _ = db.create_table(table_name);
            }
        }

        {
            let mut list = AGENTS.write().unwrap();
            if list.is_none() {
                *list = Some(HashMap::new());
            }
            let list = match &mut *list {
                Some(list) => list,
                None => {
                    eprintln!("No agent config alive, initialization of agent is failed");
                    return;
                }
            };

            list.insert(item.0.clone(), agent);
        }
    }

    for item in ids {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                tokio::time::sleep(tokio::time::Duration::new(init_sleep, 0)).await;
                println!("Starting {} agent...", item.0);
                agent::setup_agent(item.0).await;
            });
        });
    }
}

/// ## Database creator
/// 
/// This function initialize a `Database` instance and read "init_data" file if specified in config.
/// If specified it will try to parse it and upload the initial data.
/// 
/// ### Return
/// 
/// Function return with OK if "init_data" was not specified or the parse was successful.
/// In case of any parse error, function return with error.
fn initialize_db(config: &HashMap<String, String>) -> Result<(), String> {
    {
        let mut db = DB.write().unwrap();
        *db = Some(Database::new());

        if let Some(db) = &mut *db {
            db.create_table(String::from("Default")).unwrap();
        };
    }

    match config.get("init.data") {
        Some(value) => {
            println!("Read initial data from {} file", value);
            let path = std::path::Path::new(value);
            if path.exists() {
                let content = match std::fs::read_to_string(&path) {
                    Ok(content) => content,
                    Err(e) => return Err(format!("Failed to read init data file: {:?}", e)),
                };

                for line in content.lines() {
                    if line.is_empty() {
                        continue;
                    }

                    if let Err(e) = parser::parse_db_command(line) {
                        println!("Error in \"{}\" statment: {}", line, e);
                    }
                }
            }
        },
        None => println!("No init data file is specified"),
    }

    return Ok(());
}

/// ## Config reader
/// 
/// This function reads the configuration file which was specified as parameter.
/// 
/// ### Return
/// 
/// Function return with OK if file has been read and it contains the required parameters.
/// If the config file read has failed or not enough parameter is specified function return with error.
fn read_config() -> Result<HashMap<String, String>, String> {
    // Read the arguments and parse it onto a structure
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Config path must be specified as parameter!");
        return Err(String::from("Path for config is not specified"));
    }

    // Parse argument from config file
    let mut config: HashMap<String, String> = match onlyati_config::read_config(args[1].as_str()) {
        Ok(conf) => conf,
        Err(e) => return Err(format!("Error during config reading: {} {}", args[1], e)),
    };

    if let None = config.get("options.threads") {
        if let Some(n) = number_of_cores() {
            config.insert(String::from("options.threads"), n);
        }
    }

    println!("Settings:");
    for setting in &config {
        println!("- {}: {}", setting.0, setting.1);
    }

    // If some necesarry item is missign return with error
    if let Err(error) = validate_settings(&config) {
        return Err(format!("{error}"));
    }

    return Ok(config);
}

/// ## Configuration validator
/// 
/// It validate the configuration content.
fn validate_settings(settings: &HashMap<String, String>) -> Result<(), String> {
    let mut errors = String::new();

    if let None = settings.get("host.classic.address") {
        errors += "ERROR in config: Field 'address' is missing\n";
    }

    if let None = settings.get("options.threads") {
        errors += "ERROR in config: Field 'threads' is missing or couldn't fetch from /proc/cpuinfo\n";
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    return Ok(());
}

/// ## Get number of CPU cores
/// 
/// This function read the "/proc/cpuinfo" file and find the line begin wtih "cpu cores".
/// If it is found, number will be parsed and returned as value.
/// Erlse it return with none.
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
