use std::process::exit;
use std::sync::{Arc, Mutex};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod interfaces;
mod utilities;

use interfaces::classic::Classic;
use interfaces::dummy::Dummy;
use interfaces::grpc::Grpc;
use interfaces::rest::Rest;
use interfaces::ApplicationInterface;
use interfaces::InterfaceHandler;

fn main() {
    // Read RUST_LOG environment variable and set trace accordingly, default is Level::ERROR
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set loger");

    tracing::info!("hermes is initializing");

    // Build runtime
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
        tracing::error!("Configuration file is missing");
        exit(1);
    }

    let config = match onlyati_config::read_config(args[1].as_str()) {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Config file error: {}", e);
            exit(1);
        }
    };

    // Initialize HookManager for datastore
    let (hook_sender, hook_thread) = onlyati_datastore::hook::utilities::start_hook_manager();
    let hook_sender = Arc::new(Mutex::new(hook_sender));

    // Initialize Logger for datastore
    let (logger_sender, logger_thread) = match config.get("logger.location") {
        Some(path) => {
            let (sender, thread) = onlyati_datastore::logger::utilities::start_logger(path);
            let sender = Arc::new(Mutex::new(sender));
            (Some(sender), Some(thread))
        }
        None => (None, None),
    };

    // Initialize datastore
    let (sender, db_thread) = match &logger_sender {
        Some(logger_sender) => {
            let hook_sender = hook_sender.clone();
            let logger_sender = logger_sender.clone();

            onlyati_datastore::datastore::utilities::start_datastore(
                "root".to_string(),
                Some(hook_sender),
                Some(logger_sender),
            )
        }
        None => {
            let hook_sender = hook_sender.clone();
            onlyati_datastore::datastore::utilities::start_datastore(
                "root".to_string(),
                Some(hook_sender),
                None,
            )
        }
    };

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

    if let Some(logger_thread) = logger_thread {
        handler.register_interface(
            Box::new(Dummy::new(Some(logger_thread))),
            "Logger".to_string(),
        )
    }

    // Register classic interface
    if let Some(addr) = config.get("host.classic.address") {
        handler.register_interface(
            Box::new(Classic::new(sender.clone(), addr.clone())),
            "Classic".to_string(),
        )
    }

    // Register gRPC interface
    if let Some(addr) = config.get("host.grpc.address") {
        let hook_sender = hook_sender.clone();
        let logger_sender = match &logger_sender {
            Some(logger) => Some(logger.clone()),
            None => None,
        };

        handler.register_interface(
            Box::new(Grpc::new(
                sender.clone(),
                addr.clone(),
                hook_sender,
                logger_sender,
            )),
            "gRPC".to_string(),
        )
    }

    // Register REST interface
    if let Some(addr) = config.get("host.rest.address") {
        let hook_sender = hook_sender.clone();
        let logger_sender = match &logger_sender {
            Some(logger) => Some(logger.clone()),
            None => None,
        };

        handler.register_interface(
            Box::new(Rest::new(
                sender.clone(),
                addr.clone(),
                hook_sender,
                logger_sender,
            )),
            "REST".to_string(),
        )
    }

    // Start interfaces and watch them
    handler.start();
    handler.watch().await; // Block the thread, panic if service failed
}
