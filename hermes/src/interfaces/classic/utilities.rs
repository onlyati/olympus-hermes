// External dependencies
use std::sync::{mpsc::Sender, Arc, Mutex};
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

// Internal dependencies
use onlyati_datastore::datastore::{
    enums::{DatabaseAction, pair::ValueType},
    utilities,
};

// Import macros
use super::macros::{
    return_client_error, return_ok, return_ok_with_value, return_server_error, send_data_request,
};

/// Read parameters from request then execute them
pub fn parse_request(
    request: Vec<u8>,
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
) -> Result<Vec<u8>, String> {
    let valid_commands = vec!["SET", "GET", "REMKEY", "REMPATH", "LIST"];
    let request = match String::from_utf8(request) {
        Ok(req) => req,
        Err(e) => return Err(format!("failed to read request: {}", e)),
    };

    let mut command = String::new();
    let mut key = String::new();
    let mut value = String::new();
    let mut copy = 0;

    // Go through characters and parse it
    for byte in request.chars() {
        if copy == 0 {
            if byte == ' ' {
                if !valid_commands.contains(&command.as_str()) {
                    // If not valid command then don't check further
                    tracing::debug!("invalid command specified: {}", command);
                    return Err(">Err\nInvalid command\n".to_string());
                }
                copy += 1;
                continue;
            }
            command.push(byte);
        }
        if copy == 1 {
            if byte == ' ' {
                copy += 1;
                continue;
            }
            key.push(byte);
        }
        if copy == 2 {
            value.push(byte);
        }
    }

    tracing::debug!("traced paramerets: Command: {}, Key: {}, Value: {}", command, key, value);

    // Execute what the request asked then return with a reponse
    return Ok(handle_command(command, key, value, data_sender));
}

/// Requst has been parsed and this function executes what it had to
fn handle_command(
    command: String,
    key: String,
    value: String,
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
) -> Vec<u8> {
    // Key is required for all request
    if key.is_empty() {
        tracing::trace!("key is missing");
        return_client_error!("Key is missing");
    }

    match command.as_str() {
        "SET" => {
            // SET without value is an error
            if value.is_empty() {
                tracing::debug!("no value specified for SET action");
                return_client_error!("Value is missing")
            }

            // Handle SET request
            let (tx, rx) = utilities::get_channel_for_set();
            let set_action = DatabaseAction::Set(tx, key, value);
            send_data_request!(set_action, data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(_) => return_ok!(),
                    Err(e) => return_client_error!(e),
                },
                Err(e) => return_server_error!(e),
            }
        }
        "GET" => {
            // Handle GET request
            let (tx, rx) = utilities::get_channel_for_get();
            let get_action = DatabaseAction::Get(tx, key);
            send_data_request!(get_action, data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(value) => match value {
                        ValueType::RecordPointer(data) => return_ok_with_value!(data),
                        _ => return_server_error!("Pointer must be Record but it was Table"),
                    },
                    Err(e) => return_client_error!(e),
                },
                Err(e) => return_server_error!(e),
            }
        }
        "LIST" => {
            // Handle LIST request
            let (tx, rx) = utilities::get_channel_for_list();
            let list_action =
                DatabaseAction::ListKeys(tx, key, onlyati_datastore::datastore::enums::ListType::All);
            send_data_request!(list_action, data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(list) => {
                        let mut data = String::new();
                        for key in list {
                            data += key.get_key();
                            data += "\n";
                        }
                        return_ok_with_value!(data);
                    }
                    Err(e) => return_client_error!(e),
                },
                Err(e) => return_server_error!(e),
            }
        }
        "REMKEY" => {
            // Handle REMKEY request
            let (tx, rx) = utilities::get_channel_for_delete();
            let rem_action = DatabaseAction::DeleteKey(tx, key);
            send_data_request!(rem_action, data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(_) => return_ok!(),
                    Err(e) => return_client_error!(e),
                },
                Err(e) => return_server_error!(e),
            }
        }
        "REMPATH" => {
            // Handle REMPATH request
            let (tx, rx) = utilities::get_channel_for_delete();
            let rem_action = DatabaseAction::DeleteTable(tx, key);
            send_data_request!(rem_action, data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(_) => return_ok!(),
                    Err(e) => return_client_error!(e),
                },
                Err(e) => return_server_error!(e),
            }
        }
        _ => (),
    }

    return_client_error!("Invalid command");
}

/// Run Classic interface
pub async fn run_async(data_sender: Arc<Mutex<Sender<DatabaseAction>>>, address: String) {
    tracing::info!("classic interface on {} is starting...", address);

    // Try to bind for address
    let listener = match TcpListener::bind(address.clone()).await {
        Ok(listener) => listener,
        Err(e) => panic!("classic interface failed to bind: {e}"),
    };

    loop {
        // Catch every connection
        let mut socket = match listener.accept().await {
            Ok(socket) => socket,
            Err(e) => panic!("failed to accept connection: {}", e),
        };

        // Spawn thread for them
        let data_sender = data_sender.clone();
        tokio::spawn(async move {
            let mut request: Vec<u8> = Vec::with_capacity(4096);

            // Read the request
            tracing::trace!("reading request");
            loop {
                let mut buffer = BytesMut::with_capacity(4096);
                match socket.0.read_buf(&mut buffer).await {
                    // socket closed
                    Ok(n) if n == 0 => {
                        tracing::trace!("has read EOF");
                        break;
                    },
                    Ok(n) => {
                        tracing::trace!("has read {} bytes", n);
                        for byte in &buffer[0..n] {
                            request.push(*byte);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };
            }
            tracing::trace!("has been read {} bytes", request.len());

            // Handle it
            let response = match parse_request(request, data_sender) {
                Ok(vector) => String::from_utf8(vector).unwrap(),
                Err(e) => e,
            };

            tracing::trace!("write length: {}", response.len());

            // And send the response back
            if let Err(e) = socket.0.write(response.as_bytes()).await {
                tracing::warn!("failed to write to socket; err = {:?}", e);
                return;
            }
            tracing::trace!("close connection");
            let _ = socket.0.flush();
        });
    }
}
