// External depencies
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::thread::JoinHandle;

// Internal depencies
use super::ApplicationInterface;
use onlyati_datastore::datastore::enums::DatabaseAction;
use onlyati_datastore::hook::enums::HookManagerAction;
use onlyati_datastore::logger::enums::LoggerAction;

mod macros;
mod utilities;

/// Struct that handles the REST interface
pub struct Rest {
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
    hook_sender: Arc<Mutex<Sender<HookManagerAction>>>,
    logger_sender: Option<Arc<Mutex<Sender<LoggerAction>>>>,
    address: String,
    thread: Option<JoinHandle<()>>,
}

impl Rest {
    /// Create new interface
    pub fn new(
        data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
        address: String,
        hook_sender: Arc<Mutex<Sender<HookManagerAction>>>,
        logger_sender: Option<Arc<Mutex<Sender<LoggerAction>>>>,
    ) -> Self {
        return Self {
            data_sender,
            address,
            thread: None,
            hook_sender,
            logger_sender,
        };
    }
}

impl ApplicationInterface for Rest {
    fn run(&mut self) {
        let data_sender = self.data_sender.clone();
        let addres = self.address.clone();
        let hook_sender = self.hook_sender.clone();
        let logger_sender = self.logger_sender.clone();
        let thread = std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                utilities::run_async(data_sender, addres, hook_sender, logger_sender).await;
            });
        });

        self.thread = Some(thread);
    }

    fn is_it_run(&self) -> Option<bool> {
        match &self.thread {
            Some(thread) => Some(!thread.is_finished()),
            None => None,
        }
    }
}
