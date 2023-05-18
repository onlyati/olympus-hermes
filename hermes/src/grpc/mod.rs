use std::sync::{mpsc::Sender, Arc, Mutex};
use std::thread::JoinHandle;

use onlyati_datastore::enums::DatabaseAction;

use crate::traits::ApplicationInterface;

pub struct Grpc {
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
    address: String,
    thread: Option<JoinHandle<()>>,
}

mod macros;
mod utilities;

impl Grpc {
    /// Create new interface
    pub fn new(data_sender: Arc<Mutex<Sender<DatabaseAction>>>, address: String) -> Self {
        return Self {
            data_sender,
            address,
            thread: None,
        };
    }
}

impl ApplicationInterface for Grpc {
    /// Function to start the interface
    fn run(&mut self) {
        let data_sender = self.data_sender.clone();
        let addres = self.address.clone();
        let thread = std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
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
