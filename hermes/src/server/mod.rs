use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

mod interfaces;
mod utilities;

use interfaces::classic::Classic;
use interfaces::dummy::Dummy;
use interfaces::rest::Rest;
use interfaces::websocket::Websocket;
use interfaces::ApplicationInterface;
use interfaces::InterfaceHandler;

/// Main entrypoint when Hermes run as a server
///
/// # Parameters
/// - `args`: Command arguments that has been parse bly `clap`.
///
/// # Details
///
/// This function start the server by the following stpes:
/// 1. Initialize tracer
/// 1. Read configuration that path has been passed as argument
/// 1. Initialize datastore, logger and hook manager
/// 1. Register interfaces that has been enabled in the configueration file
/// 1. Register handler for interrupt and terminate signals (for graceful shutdown)
/// 1. Start registered interfaces and if any of them fails, then stop the application
///
/// # Return
///
/// This function return with a code normally. If something error would occure then with the error itself.
pub async fn main_async(args: String) -> Result<i32, Box<dyn std::error::Error>> {
    // Read environment variable and set trace accordingly, default is Level::ERROR
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("HERMES_LOG"))
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set loger");

    // Override the default panic handler that the output is written via tracer
    let _ = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        for line in info.to_string().lines() {
            tracing::error!("{}", line);
            std::process::exit(-1);
        }
    }));

    // Read configuration
    let config = match utilities::config_parse::parse_config(&args) {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Config file error: {}", e);
            return Ok(1);
        }
    };
    let config_arc = Arc::new(RwLock::new(config.clone()));

    // Initialize HookManager and Logger for Datastore)
    let (hook_sender, hook_thread) = onlyati_datastore::hook::utilities::start_hook_manager().await;
    let (logger_sender, logger_thread) = if config.general.logging {
        let path = config.logger.unwrap().location;
        let (a, b) = onlyati_datastore::logger::utilities::start_logger(&path).await;
        (Some(a), b)
    } else {
        (None, tokio::spawn(async move {}))
    };

    // Initialize Datastore
    let (sender, db_thread) = onlyati_datastore::datastore::utilities::start_datastore(
        config.general.database_name,
        Some(hook_sender),
        logger_sender,
    )
    .await;

    // Parse the input data for database and hooks too
    utilities::initial_parse::parse_initial_file(&config.initials.path, &sender)
        .await
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

    if config.general.logging {
        handler.register_interface(
            Box::new(Dummy::new(Some(logger_thread))),
            "Logger".to_string(),
        );
    }

    // Register classic interface
    if let Some(addr) = &config.network.classic {
        let config = config_arc.clone();
        handler.register_interface(
            Box::new(Classic::new(sender.clone(), addr.clone(), config)),
            "Classic".to_string(),
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

    // Register websocket interface
    if let Some(addr) = &config.network.websocket {
        let config = config_arc.clone();
        handler.register_interface(
            Box::new(Websocket::new(sender.clone(), addr.clone(), config)),
            "websocket".to_string(),
        )
    }

    // Start interfaces and watch them
    handler.start();

    // Register signal actions for termination
    tracing::debug!("register signal for termination (ctrl+c)");
    let mut terminate =
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(signal) => signal,
            Err(e) => {
                tracing::error!("failed to register terminate signal: {}", e);
                return Ok(8);
            }
        };

    tracing::debug!("register signal for interrupt (kill)");
    let mut interrupt =
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()) {
            Ok(signal) => signal,
            Err(e) => {
                tracing::error!("failed to register terminate signal: {}", e);
                return Ok(8);
            }
        };

    // Start application
    tracing::debug!("service is starting");
    tokio::select! {
        _ = handler.watch() => {
            tracing::error!("application has been stopped");
            Ok(-16)
        }
        _ = terminate.recv() => {
            tracing::info!("stop signal has recieved");
            Ok(-8)
        }
        _ = interrupt.recv() => {
            tracing::info!("interrupt signal has recieved");
            Ok(-8)
        }
    }
}
