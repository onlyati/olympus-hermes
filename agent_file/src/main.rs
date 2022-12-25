use hermes::hermes_client::{HermesClient};
use hermes::{SetPair, Pair};

use std::fs;
use std::path::Path;
use std::process::exit;

mod models;
use models::agent_config;

pub mod hermes {
    tonic::include_proto!("hermes");
}

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async move {
        main_async().await;
    });
}


async fn main_async() {
    // Measure runtime of script
    let start = std::time::Instant::now();

    // Arguments consist a list of config files, normally it is the following
    // 1. Common config
    // 2. Agent specific config
    // Exit with error if does not have enough arguments, else convert them to Path
    let mut args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Config files are not specified");
        exit(1);
    }

    args.remove(0);

    let config_paths: Vec<&Path> = args.iter()
        .map(|x| Path::new(x))
        .collect();

    // Create a new agent config, then try to parse the provided config files
    let mut config = agent_config::Config::new();

    match config.parse_config(config_paths) {
        Ok(_) => println!("Successfully processed config file!"),
        Err(e) => {
            eprintln!("Failed to read config:\n{}", e);
            exit(1);
        }
    }

    // Try to connect to Hermes via gRPC port
    println!("Connecting to Hermes via gRPC...");
    let address = config.get_hermes_address().clone().unwrap();
    let mut grpc_client = match HermesClient::connect(address).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to Hermes: {}", e);
            exit(2);
        },
    };
    println!("Connected!");

    let mut final_rc = 0;

    // Finally, read every file from config then try to set the key-value pair in Hermes
    for file in config.get_file_list() {
        let (path, key) = file.get_info();
        let table = config.get_table_name();
        let table = match table {
            Some(t) => t,
            None => {
                eprintln!("Table is not specified in config!");
                exit(2);
            }
        };

        let path = Path::new(path);
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Failed to read file: {} -> {}", path.display(), e);
                final_rc = 4;
                String::new()
            }
        };

        if content.is_empty() {
            continue;
        }

        let pair = SetPair {
            key: key.to_string(),
            table: table.to_string(),
            value: content,
        };

        let request = tonic::Request::new(pair);
        let response: Result<tonic::Response<Pair>, tonic::Status> = grpc_client.set(request).await;

        if let Err(e) = response {
            eprintln!("Failed to update Hermes: {:?}", e);
            final_rc = 4;
        }

        println!("File has been processed: {} -> {} {}", path.display(), table, key);
    }

    let elapsed = start.elapsed();
    println!("Agent is ended under {:?} time.", elapsed);
    exit(final_rc);
}
