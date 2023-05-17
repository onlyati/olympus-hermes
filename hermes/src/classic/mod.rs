use std::sync::{mpsc::Sender, Arc, Mutex};
use std::thread::JoinHandle;

use bytes::BytesMut;
use onlyati_datastore::enums::DatabaseAction;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::traits::ApplicationInterface;

mod utilities;
mod macros;

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

    pub async fn run_async(data_sender: Arc<Mutex<Sender<DatabaseAction>>>, address: String) {
        let listener = match TcpListener::bind(address.clone()).await {
            Ok(listener) => listener,
            Err(e) => panic!("Classic interface failed to bind: {e}"),
        };

        println!("Start listening on {}", address);

        loop {
            let mut socket = match listener.accept().await {
                Ok(socket) => socket,
                Err(e) => panic!("Failed to accept connection: {}", e),
            };

            let data_sender = data_sender.clone();
            tokio::spawn(async move {
                let mut request: Vec<u8> = Vec::with_capacity(8);

                loop {
                    let mut buffer = BytesMut::with_capacity(8);
                    match socket.0.read_buf(&mut buffer).await {
                        // socket closed
                        Ok(n) if n == 0 => break,
                        Ok(n) => {
                            for byte in &buffer[0..n] {
                                request.push(*byte);
                            }
                        }
                        Err(e) => {
                            eprintln!("failed to read from socket; err = {:?}", e);
                            return;
                        }
                    };
                }

                let response = match utilities::parse_request(request, data_sender) {
                    Ok(vector) => String::from_utf8(vector).unwrap(),
                    Err(e) => e,
                };

                // Write the data back
                if let Err(e) = socket.0.write_all(response.as_bytes()).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            });
        }
    }
}

impl ApplicationInterface for Classic {
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
                Classic::run_async(data_sender, addres).await;
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
