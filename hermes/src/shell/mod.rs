use clap::Parser;
use futures_util::SinkExt;
use rustyline::{config::Configurer, error::ReadlineError};
use rustyline::{CompletionType, DefaultEditor};
use termion::color::{self, Fg};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};

use crate::arg::ShellParms;
use crate::common::websocket::client::{connecto_to_server, perform_action};

mod utilities;

const LIGHT_BLUE: Fg<color::LightBlue> = Fg(color::LightBlue);
const LIGHT_GREEN: Fg<color::LightGreen> = Fg(color::LightGreen);
const RED: Fg<color::Red> = Fg(color::Red);
const LIGHT_YELLOW: Fg<color::LightYellow> = Fg(color::LightYellow);
const WHITE: Fg<color::White> = Fg(color::White);
const LIGHT_GRAY: Fg<color::LightBlack> = Fg(color::LightBlack);

pub async fn main_async(mut hostname: Option<String>) -> Result<i32, Box<dyn std::error::Error>> {
    // Read environment variable and set trace accordingly, default is Level::ERROR
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("HERMES_CLI_LOG"))
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set loger");

    let mut stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>> = None;

    if let Some(hostname) = &hostname {
        stream = match connecto_to_server(hostname.clone()).await {
            Ok(stream) => Some(stream),
            Err(e) => {
                for line in e.lines() {
                    println!("{}{}", RED, line);
                }
                return Ok(1);
            }
        };
    }

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

    loop {
        let prefix = if stream.is_some() && hostname.is_some() {
            let hostname = hostname.clone();
            format!("{}hermes@{}=> {}", LIGHT_GREEN, hostname.unwrap(), WHITE)
        } else {
            format!("{}hermes@disconnected=> {}", LIGHT_GRAY, WHITE)
        };

        let readline = rl.readline(&prefix);
        match readline {
            Ok(line) => {
                match line.as_str() {
                    "\\q" => break,
                    "\\clear" => {
                        std::process::Command::new("clear").status().unwrap();
                    }
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
                    "\\?" => {
                        println!("{}Hermes shell commands:", LIGHT_BLUE);
                        println!("{}\\c ws://host:port   - Connect to a Hermes", LIGHT_BLUE);
                        println!("{}\\d                  - Disconnect", LIGHT_BLUE);
                        println!("{}\\clear              - Clear screen", LIGHT_BLUE);
                        println!("{}\\q                  - Quit", LIGHT_BLUE);
                    }
                    ref line if line.starts_with("\\c ") => {
                        if let Some(socket) = &mut stream {
                            if let Err(e) = socket.send(Message::Close(None)).await {
                                println!("{}{}", RED, e);
                            }
                            stream = None;
                        }

                        let address = line.split_whitespace().collect::<Vec<&str>>();
                        if address.len() == 0 {
                            println!("{}No address is specified", RED);
                        }
                        hostname = Some(address.last().unwrap().to_string());

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
                    line => {
                        // This split is not fine, multiple words split cause issue
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
            Err(ReadlineError::Interrupted) => break,
            Err(err) => {
                tracing::error!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(&history_file)
        .expect("failed to save history");

    if let Some(stream) = &mut stream {
        if let Err(e) = stream.send(Message::Close(None)).await {
            tracing::error!("{}", e);
            return Ok(8);
        }
    }

    return Ok(0);
}
