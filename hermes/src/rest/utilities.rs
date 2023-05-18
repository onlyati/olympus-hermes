use std::sync::{mpsc::Sender, Arc, Mutex};

use onlyati_datastore::{enums::DatabaseAction, enums::ValueType, utilities};

pub async fn run_async(data_sender: Arc<Mutex<Sender<DatabaseAction>>>, address: String) {
    println!("REST interface on {} is starting...", address);
}