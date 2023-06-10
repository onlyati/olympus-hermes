use std::sync::{Arc, Mutex, RwLock};

mod interfaces;
mod utilities;

use interfaces::classic::Classic;
use interfaces::dummy::Dummy;
use interfaces::grpc::Grpc;
use interfaces::rest::Rest;
use interfaces::ApplicationInterface;
use interfaces::InterfaceHandler;

pub async fn main_async(args: String) -> Result<i32, Box<dyn std::error::Error>> {
    // Read environment variable and set trace accordingly, default is Level::ERROR
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("HERMES_LOG"))
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set loger");

    // Read configuration
    let config = match utilities::config_parse::parse_config(&args) {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Config file error: {}", e);
            return Ok(1);
        }
    };
    let config_arc = Arc::new(RwLock::new(config.clone()));

    // Initialize HookManager and Logger for Datastore
    let (hook_sender, hook_thread) = onlyati_datastore::hook::utilities::start_hook_manager();
    let (logger_sender, logger_thread) =
        onlyati_datastore::logger::utilities::start_logger(&config.logger.location);

    // Initialize Datastore
    let (sender, db_thread) = onlyati_datastore::datastore::utilities::start_datastore(
        "root".to_string(),
        Some(hook_sender),
        Some(logger_sender),
    );

    // Parse the input data for database and hooks too
    utilities::initial_parse::parse_initial_file(&config.initials.path, &sender)
        .unwrap_or_else(|x| panic!("{}", x));
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

    handler.register_interface(
        Box::new(Dummy::new(Some(logger_thread))),
        "Logger".to_string(),
    );

    // Register classic interface
    if let Some(addr) = &config.network.classic {
        let config = config_arc.clone();
        handler.register_interface(
            Box::new(Classic::new(sender.clone(), addr.clone(), config)),
            "Classic".to_string(),
        )
    }

    // Register gRPC interface
    if let Some(addr) = &config.network.grpc {
        let config = config_arc.clone();
        handler.register_interface(
            Box::new(Grpc::new(sender.clone(), addr.clone(), config)),
            "gRPC".to_string(),
        )
    }

    // Register REST interface
    if let Some(addr) = &config.network.rest {
        let config = config_arc.clone();
        handler.register_interface(
            Box::new(Rest::new(sender.clone(), addr.clone(), config)),
            "REST".to_string(),
        )
    }

    // Start interfaces and watch them
    handler.start();
    handler.watch().await; // Block the thread, panic if service failed

    return Ok(-8);
}