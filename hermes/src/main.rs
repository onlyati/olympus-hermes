use std::process::exit;
use std::sync::{Arc, Mutex};

mod interfaces;
mod utilities;

use interfaces::classic::Classic;
use interfaces::dummy::Dummy;
use interfaces::grpc::Grpc;
use interfaces::rest::Rest;
use interfaces::ApplicationInterface;
use interfaces::InterfaceHandler;

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

    // Initialize datastore-rs
    let (sender, hook_thread) = onlyati_datastore::hook::utilities::start_hook_manager();
    let (sender, db_thread) =
        onlyati_datastore::datastore::utilities::start_datastore("root".to_string(), Some(sender));
        
    // Parse the input data for database and hooks too
    utilities::parse_input_data("init.data", &config, &sender).unwrap_or_else(|x| panic!("{}", x));
    utilities::parse_input_hook("hook.data", &config, &sender).unwrap_or_else(|x| panic!("{}", x));
    let sender = Arc::new(Mutex::new(sender));

    // Create interface handler
    let mut handler: InterfaceHandler<Box<dyn ApplicationInterface>> = InterfaceHandler::new();

    // Register the monitor only interfaces
    handler.register_interface(
        Box::new(Dummy::new(Some(hook_thread))),
        "HookManager".to_string(),
    );

    handler.register_interface(
        Box::new(Dummy::new(Some(db_thread))),
        "Datastore".to_string(),
    );

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


