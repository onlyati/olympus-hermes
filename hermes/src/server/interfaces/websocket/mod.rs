// External dependencies
use std::sync::RwLock;
use std::sync::{mpsc::Sender, Arc, Mutex};
use tokio::task::JoinHandle;

// Internal dependencies
use super::ApplicationInterface;
use crate::server::utilities::config_parse::Config;
use onlyati_datastore::datastore::enums::DatabaseAction;

mod macros;
mod utilities;
mod structs;

// gRPC interface that run the function
pub struct Websocket {
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
    address: String,
    thread: Option<JoinHandle<()>>,
    config: Arc<RwLock<Config>>,
}

impl Websocket {
    /// Create new interface
    pub fn new(
        data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
        address: String,
        config: Arc<RwLock<Config>>,
    ) -> Self {
        return Self {
            data_sender,
            address,
            thread: None,
            config,
        };
    }
}

impl ApplicationInterface for Websocket {
    /// Function to start the interface
    fn run(&mut self) {
        let data_sender = self.data_sender.clone();
        let addres = self.address.clone();
        let config = self.config.clone();
        let thread = tokio::spawn(async move {
            utilities::run_async(data_sender, addres, config).await;
        });

        self.thread = Some(thread);
    }

    /// Check function that interface is running
    fn is_it_run(&self) -> Option<bool> {
        match &self.thread {
            Some(thread) => Some(!thread.is_finished()),
            None => None,
        }
    }
}
