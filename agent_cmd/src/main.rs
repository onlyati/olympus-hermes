use hermes::hermes_client::{HermesClient};
use hermes::{SetPair, Pair};

use std::process::exit;
use std::path::Path;

mod models;
use models::agent_config;
use models::cmd_executor;

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
    for cmd in config.get_cmd_list() {
        let (cmd_bin, args, table, key) = cmd.get_info();

        let output = cmd_executor::execute(cmd_bin.clone(), args.clone());

        let value: String = match output {
            Ok(output) => {
                let mut value = String::new();
                for line in output {
                    value += format!("{} {} {}", line.time, line.out_type, line.text).as_str();
                }
                value
            },
            Err(e) => {
                println!("Failed to execute command, exit code is {:?}", e.exit_code);
                let mut value = String::new();
                for line in e.output {
                    value += format!("{} {} {}", line.time, line.out_type, line.text).as_str();
                }
                final_rc = 4;
                value
            },
        };

        let pair = SetPair {
            key: key.clone(),
            table: table.clone(),
            value: value,
        };

        let request = tonic::Request::new(pair);
        let response: Result<tonic::Response<Pair>, tonic::Status> = grpc_client.set(request).await;

        if let Err(e) = response {
            eprintln!("Failed to update Hermes: {:?}", e);
            final_rc = 4;
        }

        println!("Command has been processed: {} {:?} -> {} {}", cmd_bin, args, table, key);
    }

    let elapsed = start.elapsed();
    println!("Agent is ended under {:?} time.", elapsed);
    exit(final_rc);
}

