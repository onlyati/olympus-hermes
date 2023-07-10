use std::path::Path;
use tokio::select;
use tokio::sync::mpsc::{channel, Sender};
use tokio::task::JoinHandle;

pub(crate) mod internal;

use super::{
    enums::{LoggerAction, LoggerResponse},
    LoggerManager,
};

/// Start logger thread
///
/// # Arguments
/// 1. `path`: directory where logger can put its files
///
/// # Return
///
/// With the thread handler and sender that can communicate with logger.
pub async fn start_logger(path: Option<String>) -> (Sender<LoggerAction>, JoinHandle<()>) {
    let (tx, mut rx) = channel::<LoggerAction>(60000);

    let path = path.to_owned();

    let thread = tokio::spawn(async move {
        let mut logger = LoggerManager::new(path);

        loop {
            select! {
                request = rx.recv() => {
                    match request {
                        Some(request) => {
                            tracing::trace!("request has come: {}", request);
                            match request {
                                LoggerAction::Resume(sender) => match logger.resume() {
                                    Ok(_) => send_response!(sender, LoggerResponse::Ok),
                                    Err(e) => send_response!(sender, LoggerResponse::Err(e)),
                                },
                                LoggerAction::Suspend(sender) => match logger.suspend() {
                                    Ok(_) => send_response!(sender, LoggerResponse::Ok),
                                    Err(e) => send_response!(sender, LoggerResponse::Err(e)),
                                },
                                LoggerAction::ReadAppendFile(sender) => {
                                    if logger.path.is_empty() {
                                        send_response!(sender, LoggerResponse::FromAppendFile(vec![]));
                                        continue;
                                    }

                                    match super::utilities::internal::read_append_file(Path::new(&format!("{}/hermes.af", logger.path))) {
                                        Ok(list) => {
                                            send_response!(sender, LoggerResponse::FromAppendFile(list))
                                        }
                                        Err(e) => send_response!(sender, LoggerResponse::Err(e)),
                                    }
                                }
                                LoggerAction::Write(sender, lines) => {
                                    for line in lines {
                                        logger.write_buffer.push_back(line);
                                    }

                                    if let Err(e) = logger.write_append_file() {
                                        tracing::error!("{}", e);
                                        send_response!(sender, LoggerResponse::Err(e));
                                        return;
                                    }

                                    send_response!(sender, LoggerResponse::Ok);
                                }
                                LoggerAction::WriteAsync(lines) => {
                                    for line in lines {
                                        logger.write_buffer.push_back(line);
                                    }

                                    if logger.write_buffer.len() > 50 {
                                        if let Err(e) = logger.write_append_file() {
                                            tracing::error!("{}", e);
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                        None => {
                            tracing::error!("failed to receive");
                            return;
                        },
                    }
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {
                    if !logger.write_buffer.is_empty() {
                        if let Err(e) = logger.write_append_file() {
                            tracing::error!("{}", e);
                            return;
                        }
                    }
                }
            }
        }
    });

    (tx, thread)
}

macro_rules! send_response {
    ($sender:expr, $value:expr) => {
        $sender
            .send($value)
            .await
            .unwrap_or_else(|e| tracing::error!("Error during send: {}", e))
    };
}
pub(self) use send_response;
