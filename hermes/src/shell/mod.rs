use clap::Parser;
use futures_util::SinkExt;
use rustyline::{config::Configurer, error::ReadlineError};
use rustyline::{CompletionType, DefaultEditor};
use termion::color::{self, Fg};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};

use crate::arg::ShellParms;
use crate::common::websocket::client::{connecto_to_server, perform_action, client_config_parse, get_address_for_client};

mod utilities;

const LIGHT_BLUE: Fg<color::LightBlue> = Fg(color::LightBlue);
const LIGHT_GREEN: Fg<color::LightGreen> = Fg(color::LightGreen);
const RED: Fg<color::Red> = Fg(color::Red);
const LIGHT_YELLOW: Fg<color::LightYellow> = Fg(color::LightYellow);
const WHITE: Fg<color::White> = Fg(color::White);
const LIGHT_GRAY: Fg<color::LightBlack> = Fg(color::LightBlack);

/// Main entrypoint for shell
/// 
/// # Parameters
/// - `hostname`: If this is specified then shell try to connect there automatically
/// - `client_config`: Client configuration file that contains pre-defined servers
/// 
pub async fn main_async(mut hostname: Option<String>, client_config: String) -> Result<i32, Box<dyn std::error::Error>> {
    // Read environment variable and set trace accordingly, default is Level::ERROR
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("HERMES_CLI_LOG"))
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set loger");

    // Check for client_config and upload nodes if exists
    let config = match client_config_parse(&client_config) {
        Ok(cfg) => Some(cfg),
        Err(e) => {
            tracing::debug!("{}", e);
            None
        }
    };

    // If hostname is specified try to connect
    let mut stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>> = None;

    if let Some(address) = &hostname {
        if address.starts_with("ws://") {
            stream = match connecto_to_server(address.clone()).await {
                Ok(stream) => Some(stream),
                Err(e) => {
                    for line in e.lines() {
                        println!("{}{}", RED, line);
                    }
                    hostname = None;
                    None
                }
            };
        }
        else if address.starts_with("cfg://") {
            if let Some(cfg) = &config {
                match get_address_for_client(address.to_string(), &cfg) {
                    Some(address) => {
                        stream = match connecto_to_server(address.clone()).await {
                            Ok(stream) => Some(stream),
                            Err(e) => {
                                for line in e.lines() {
                                    println!("{}{}", RED, line);
                                }
                                hostname = None;
                                None
                            }
                        };
                    },
                    None => {
                        println!("{}Specified server does not exist in config", RED);
                        hostname = None;
                        stream = None;
                    },
                }
            }
        }
    }

    // Initialize terminal
    let mut rl = DefaultEditor::new()?;
    rl.set_completion_type(CompletionType::List);
    rl.set_auto_add_history(true);
    let home_dir = match std::env::var("HOME") {
        Ok(var) => var,
        Err(e) => {
            println!("{}{}", RED, e);
            return Ok(-1);
        }
    };

    let history_file = &format!("{}/.hermes-shell", home_dir);
    let _ = rl.load_history(&history_file);

    // Start to receive commands from user
    loop {
        // Decide what will be the prefix at the beginning of shell
        let prefix = if stream.is_some() && hostname.is_some() {
            let hostname = hostname.clone();
            format!("{}hermes@{}=> {}", LIGHT_GREEN, hostname.unwrap(), WHITE)
        } else {
            format!("{}hermes@disconnected=> {}", LIGHT_GRAY, WHITE)
        };

        let readline = rl.readline(&prefix);

        // Check what the user typoed then decide what to do
        match readline {
            Ok(line) => {
                match line.as_str() {
                    // If empty answer, just go further
                    "" => continue,
                    //
                    // Command to quit from the shell
                    //
                    "\\q" => break,
                    //
                    // Clear the screen
                    //
                    "\\clear" => {
                        std::process::Command::new("clear").status().unwrap();
                    }
                    //
                    // Disconnect from the current Hermes instance
                    //
                    "\\d" => match &mut stream {
                        Some(socket) => {
                            if let Err(e) = socket.send(Message::Close(None)).await {
                                println!("{}{}", RED, e);
                            }
                            stream = None;
                            hostname = None;
                        }
                        None => println!("{}already does not connected", RED),
                    },
                    //
                    // List instances from client config file
                    //
                    "\\l" => {
                        match &config {
                            Some(cfg) => {
                                for node in &cfg.node {
                                    println!("{}{:20}{}", LIGHT_BLUE, node.name, node.address);
                                }
                            },
                            None => println!("{}config file does not specified or exist at shell startup", RED),
                        }
                    }
                    //
                    // Display shell help
                    //
                    "\\?" => {
                        println!("{}Hermes shell commands:", LIGHT_BLUE);
                        println!("{}\\c protocol://host:port   - Connect to a Hermes", LIGHT_BLUE);
                        println!("{}\\d                        - Disconnect", LIGHT_BLUE);
                        println!("{}\\l                        - List nodes from client config", LIGHT_BLUE);
                        println!("{}\\clear                    - Clear screen", LIGHT_BLUE);
                        println!("{}\\q                        - Quit", LIGHT_BLUE);
                    }
                    //
                    // Command to connect to Hermes instance
                    //
                    ref line if line.starts_with("\\c ") => {
                        // Disconnect from the current one if already connected
                        if let Some(socket) = &mut stream {
                            if let Err(e) = socket.send(Message::Close(None)).await {
                                println!("{}{}", RED, e);
                            }
                            stream = None;
                        }

                        // Gather address that the user typed
                        let address = line.split_whitespace().collect::<Vec<&str>>();
                        if address.len() != 2 {
                            println!("{}invalid syntax, proper one: \\c HOSTNAME", RED);
                            continue;
                        }
                        let mut temp_host = address[1].to_string();

                        // If this is a cfg:// then resolve the address
                        if temp_host.starts_with("cfg://") {
                            match &config {
                                Some(cfg) => match get_address_for_client(temp_host.clone(), &cfg) {
                                    Some(addr) => temp_host = addr,
                                    None => {
                                        println!("{}specified server does not found in client config list", RED);
                                        hostname = None;
                                        continue;
                                    }
                                },
                                None => {
                                    println!("{}cfg:// is specified but no config list available", RED);
                                    hostname = None;
                                    continue;
                                },
                            }
                        }

                        // Try to connect
                        hostname = Some(temp_host);
                        stream =
                            match connecto_to_server(hostname.clone().unwrap())
                                .await
                            {
                                Ok(stream) => Some(stream),
                                Err(e) => {
                                    for line in e.lines() {
                                        println!("{}{}", RED, line);
                                    }
                                    continue;
                                }
                            };
                    }
                    //
                    // Process Hermes command
                    //
                    line => {
                        // Split the parameters then parse it with clap
                        let words = utilities::args::split_arguments(line);
                        let args = match ShellParms::try_parse_from(words) {
                            Ok(args) => args,
                            Err(e) => {
                                for line in e.to_string().lines() {
                                    println!("{}{}", LIGHT_YELLOW, line);
                                }
                                continue;
                            }
                        };

                        // If connected to a Hermes instance then execute command and show output respectively its success
                        if let Some(stream) = &mut stream {
                            match perform_action(stream, args.action).await {
                                Ok(response) => {
                                    for line in response.lines() {
                                        println!("{}{}", LIGHT_BLUE, line);
                                    }
                                }
                                Err(e) => {
                                    for line in e.lines() {
                                        println!("{}{}", RED, line);
                                    }
                                }
                            };
                        } else {
                            println!("{}not connected to a server", RED);
                        }
                    }
                }
            }
            //
            // If Ctrl+C is pressed then exit
            //
            Err(ReadlineError::Interrupted) => break,
            //
            // If Ctrl+D is pressed then exit
            //
            Err(ReadlineError::Eof) => break,
            //
            // For other errors, write them down
            //
            Err(err) => {
                tracing::error!("Error: {:?}", err);
                break;
            }
        }
    }

    // Save command history
    rl.save_history(&history_file)
        .expect("failed to save history");

    // Close connection with hermes if it exists
    if let Some(stream) = &mut stream {
        if let Err(e) = stream.send(Message::Close(None)).await {
            tracing::error!("{}", e);
            return Ok(8);
        }
    }

    return Ok(0);
}
