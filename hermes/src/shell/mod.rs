use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Editor};
use serde::{Deserialize, Serialize};
use std::io::{stdin, stdout};
use std::io::{Read, Write};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::arg::ShellParms;

pub async fn main_async(mut hostname: String) -> Result<i32, Box<dyn std::error::Error>> {
    // Read environment variable and set trace accordingly, default is Level::ERROR
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("HERMES_CLI_LOG"))
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set loger");

    hostname += "/ws";
    let url = match url::Url::parse(&hostname[..]) {
        Ok(url) => url,
        Err(e) => {
            tracing::error!("{}", e);
            return Ok(1);
        }
    };

    let (mut stream, _) = connect_async(url).await.expect("failed to connect");

    let mut rl = DefaultEditor::new()?;
    let home_dir = match std::env::var("HOME") {
        Ok(var) => var,
        Err(e) => {
            tracing::error!("{}", e);
            return Ok(-1);
        }
    };

    let prefix = format!("hermes=> ");
    let history_file = &format!("{}/.hermes-shell", home_dir);
    let _ = rl.load_history(&history_file);

    loop {
        let readline = rl.readline(&prefix);
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())
                    .expect("failed to add history");

                match line.as_str() {
                    "\\q" => break,
                    "\\clear" => {
                        std::process::Command::new("clear").status().unwrap();
                    },
                    "\\?" => {
                        println!("Hermes shell commands:");
                        println!("\\clear   - Clear screen");
                        println!("\\q       - Quit");
                    },
                    line => {
                        // This split is not fine, multiple words split cause issue
                        let mut words = line
                            .split_whitespace()
                            .map(|word| String::from(word))
                            .collect::<Vec<String>>();
                        words.insert(0, "hermes".to_string());
                        let args = match ShellParms::try_parse_from(words) {
                            Ok(args) => args,
                            Err(e) => {
                                for line in e.to_string().lines() {
                                    println!("{}", line);
                                }
                                continue;
                            }
                        };
                        println!("{:?}", args);
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

    if let Err(e) = stream.send(Message::Close(None)).await {
        tracing::error!("{}", e);
        return Ok(8);
    }

    return Ok(0);
}
