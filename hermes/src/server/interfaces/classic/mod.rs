use std::sync::RwLock;
// External dependencies
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::thread::JoinHandle;

use crate::server::utilities::config_parse::Config;

// Internal dependecies
use super::ApplicationInterface;
use onlyati_datastore::datastore::enums::DatabaseAction;

mod macros;
mod utilities;

/// Classic interface that run functions
/// Functions:
/// - SET `key` `value`
/// - GET `key`
/// - REMKEY `key`
/// - REMPATH `key`
/// - LIST `key`
/// - TRIGGER `key` `value`
/// - GETHOOK `key`
/// - SETHOOK `key` `link`
/// - REMHOOK `key` `link`
/// - LISTHOOK `key`
/// - SUSPEND LOG
/// - RESUME LOG
/// - EXEC `key` `script` `set or trigger` `value`
pub struct Classic {
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
    address: String,
    thread: Option<JoinHandle<()>>,
    config: Arc<RwLock<Config>>,
}

impl Classic {
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

impl ApplicationInterface for Classic {
    /// Function to start the interface
    fn run(&mut self) {
        let data_sender = self.data_sender.clone();
        let addres = self.address.clone();
        let config = self.config.clone();
        let thread = std::thread::spawn(move || {
            tracing::trace!("allocate new multi threaded runtime to classic interface");
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                tracing::trace!("Start classic interface");
                utilities::run_async(data_sender, addres, config).await;
            });
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
