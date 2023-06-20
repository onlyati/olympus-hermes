use std::{
    sync::mpsc::{self, Sender},
    thread::JoinHandle,
};

use crate::logger::enums::LogState;

use super::{
    enums::{LoggerAction, LoggerResponse},
    LoggerManager,
};

pub fn start_logger(path: &str) -> (Sender<LoggerAction>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel::<LoggerAction>();

    let path = path.to_owned();

    let thread = std::thread::spawn(move || {
        let mut logger = LoggerManager::new(path);

        while let Ok(request) = rx.recv() {
            tracing::trace!("request has come: {}", request);
            match request {
                LoggerAction::Resume(sender) => match logger.resume() {
                    Ok(_) => send_response!(sender, LoggerResponse::Ok),
                    Err(e) => send_response!(sender, LoggerResponse::Err(e)),
                },
                LoggerAction::Suspend(sender) => match logger.suspend() {
                    Ok(_) => send_response!(sender, LoggerResponse::Ok),
                    Err(e) => send_response!(sender, LoggerResponse::Err(e)),
                }
                LoggerAction::Write(sender, lines) => {
                    if logger.state != LogState::Suspended {
                        if let Err(e) = logger.start()  {
                            tracing::error!("failed to start logging: {}", e);
                            send_response!(sender, LoggerResponse::Err(e));
                            continue;
                        }
                    }

                    for line in lines {
                        if let Err(e) = logger.write(line) {
                            tracing::error!("failed to write logging: {}", e);
                            send_response!(sender, LoggerResponse::Err(e));
                            continue;
                        }
                    }

                    if logger.state != LogState::Suspended {
                        if let Err(e) = logger.stop() {
                            tracing::error!("failed to stop logging: {}", e);
                            send_response!(sender, LoggerResponse::Err(e));
                        }
                    }

                    send_response!(sender, LoggerResponse::Ok);
                }
                LoggerAction::WriteAsync(lines) => {
                    if logger.state != LogState::Suspended {
                        if let Err(e) = logger.start() {
                            tracing::error!("failed to start logging: {}", e);
                            continue;
                        }
                    }

                    for line in lines {
                        if let Err(e) = logger.write(line) {
                            tracing::error!("failed to write logging: {}", e);
                            continue;
                        }
                    }

                    if logger.state != LogState::Suspended {
                        if let Err(e) = logger.stop() {
                            tracing::error!("failed to stop logging: {}", e);
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
            .unwrap_or_else(|e| tracing::error!("Error during send: {}", e))
    };
}
pub(self) use send_response;
