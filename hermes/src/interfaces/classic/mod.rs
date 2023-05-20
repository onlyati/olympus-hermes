// External dependencies
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::thread::JoinHandle;

// Internal dependecies
use onlyati_datastore::datastore::enums::DatabaseAction;
use super::ApplicationInterface;

mod macros;
mod utilities;

/// Classic interface that run functions
/// Functions:
/// - SET `key` `value`
/// - GET `key`
/// - REMKEY `key`
/// - REMPATH `key`
/// - LIST `key`
pub struct Classic {
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
    address: String,
    thread: Option<JoinHandle<()>>,
}

impl Classic {
    /// Create new interface
    pub fn new(data_sender: Arc<Mutex<Sender<DatabaseAction>>>, address: String) -> Self {
        return Self {
            data_sender,
            address,
            thread: None,
        };
    }
}

impl ApplicationInterface for Classic {
    /// Function to start the interface
    fn run(&mut self) {
        let data_sender = self.data_sender.clone();
        let addres = self.address.clone();
        let thread = std::thread::spawn(move || {
            tracing::trace!("allocate new multi threaded runtime to classic interface");
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                tracing::trace!("Start classic interface");
                utilities::run_async(data_sender, addres).await;
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
