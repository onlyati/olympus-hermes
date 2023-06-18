use crate::arg::CliArgs;
use crate::common::websocket::client::{get_config, get_address_for_client};

/// Entrypoint of cli
/// 
/// # Parameters
/// - `args`: Arguments that has been parsed
/// 
/// # Return
/// 
/// In case of internal error with an error.
/// Else return codes mean the following:
/// - 0: Everything was fine
/// - 1: Could not connect to server something wrong with server address parametersr
/// - 2: Request has been sent to server that has processed it, but something was wrong with passed parameters
/// 
pub async fn main_async(args: CliArgs) -> Result<i32, Box<dyn std::error::Error>> {
    // Read environment variable and set trace accordingly, default is Level::ERROR
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("HERMES_CLI_LOG"))
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set loger");

    // Get the host name, if cfg:// is specified then read from config
    let hostname = if args.hostname.starts_with("cfg://") {
        tracing::trace!("config is selected: {}", args.hostname);
        let config = match get_config(&args.config) {
            Ok(cfg) => cfg,
            Err(e) => {
                println!(">Error\n{}", e);
                return Ok(1);
            }
        };

        match get_address_for_client(args.hostname, &config) {
            Some(addr) => addr,
            None => {
                println!(">Error\nSpecified config does not found in file");
                return Ok(1);
            }
        }
    } else {
        args.hostname
    };

    // Try to connect to server
    let mut stream = match crate::common::websocket::client::connecto_to_server(hostname).await {
        Ok(stream) => stream,
        Err(e) => {
            println!(">Error\n{}", e);
            return Ok(1);
        }
    };

    let start = std::time::Instant::now();

    // Perform the requested action
    match crate::common::websocket::client::perform_action(&mut stream, args.action).await {
        Ok(resp) => println!(">Ok\n{}", resp),
        Err(e) => {
            println!(">Error\n{}", e);
            return Ok(2);
        }
    }

    let elapsed = start.elapsed();
    tracing::debug!("Measured runtime: {:?}", elapsed);

    return Ok(0);
}
