use std::collections::HashMap;
use std::path::Path;
use std::process::exit;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

mod interfaces;

use interfaces::classic::Classic;
use interfaces::grpc::Grpc;
use interfaces::rest::Rest;
use interfaces::ApplicationInterface;
use interfaces::InterfaceHandler;
use onlyati_datastore::enums::DatabaseAction;
use onlyati_datastore::utilities;

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
    // Read configuration
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Configuration file is missing");
        exit(1);
    }

    let config = match onlyati_config::read_config(args[1].as_str()) {
        Ok(config) => config,
        Err(e) => panic!("Config file error: {}", e),
    };

    // Start datastore thread
    let sender = onlyati_datastore::utilities::start_datastore("root".to_string());
    parse_input_data("init.data", &config, &sender).unwrap_or_else(|x| panic!("{}", x));
    let sender = Arc::new(Mutex::new(sender));

    // Create interface handler
    let mut handler: InterfaceHandler<Box<dyn ApplicationInterface>> = InterfaceHandler::new();

    // Register classic interface
    if let Some(addr) = config.get("host.classic.address") {
        handler.register_interface(
            Box::new(Classic::new(sender.clone(), addr.clone())),
            "Classic".to_string(),
        )
    }

    // Register gRPC interface
    if let Some(addr) = config.get("host.grpc.address") {
        handler.register_interface(
            Box::new(Grpc::new(sender.clone(), addr.clone())),
            "gRPC".to_string(),
        )
    }

    // Register REST interface
    if let Some(addr) = config.get("host.rest.address") {
        handler.register_interface(
            Box::new(Rest::new(sender.clone(), addr.clone())),
            "REST".to_string(),
        )
    }

    // Start interfaces and watch them
    handler.start();
    handler.watch().await; // Block the thread, panic if service failed
}


/// Parse the input file and upload onto database before anything would happen
fn parse_input_data(
    setting_name: &str,
    config: &HashMap<String, String>,
    data_sender: &Sender<DatabaseAction>,
) -> Result<(), String> {
    if let Some(path) = config.get(setting_name) {
        let path = Path::new(path);
        if path.exists() {
            // Read the file
            let content = match std::fs::read_to_string(path) {
                Ok(info) => info,
                Err(e) => {
                    return Err(format!(
                        "Error: Could not read file: {} {}",
                        path.display(),
                        e
                    ))
                }
            };

            // Find where ends the key and where the value begin
            for line in content.lines() {
                if line.is_empty() {
                    continue;
                }

                if &line[0..1] == " " {
                    continue;
                }

                let mut end_of_key: usize = 0;
                let mut start_of_value: usize = 0;
                let mut index: usize = 0;
                for char in line.chars() {
                    if char == ' ' && end_of_key == 0 {
                        end_of_key = index;
                        continue;
                    }

                    if char != ' ' && end_of_key != 0 {
                        start_of_value = index + 1;
                        break;
                    }

                    index += 1;
                }

                // Allocate strings
                let key = String::from(&line[0..end_of_key]);
                let value = String::from(&line[start_of_value..]);

                // Then upload onto database
                let (tx, rx) = utilities::get_channel_for_set();
                let action = DatabaseAction::Set(tx, key, value);

                if let Err(e) = data_sender.send(action) {
                    return Err(format!("Error: {}", e));
                }

                match rx.recv() {
                    Ok(response) => match response {
                        Err(e) => return Err(format!("Error: {}", e)),
                        _ => (),
                    },
                    Err(e) => return Err(format!("Error: {}", e)),
                }
            }
        } else {
            return Err(format!(
                "Error: Specified file does not exist: {}",
                path.display()
            ));
        }
    }

    return Ok(());
}
