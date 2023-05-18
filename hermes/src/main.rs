use std::process::exit;
use std::sync::{Arc, Mutex};

mod classic;
mod grpc;
mod rest;
mod interface_handler;
mod traits;

use classic::Classic;
use grpc::Grpc;
use interface_handler::InterfaceHandler;
use rest::Rest;
use traits::ApplicationInterface;

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
