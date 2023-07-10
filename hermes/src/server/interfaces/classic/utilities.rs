// External dependencies
use bytes::BytesMut;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{mpsc::channel, mpsc::Sender, Mutex, RwLock};

// Internal dependencies
use onlyati_datastore::datastore::enums::{pair::ValueType, DatabaseAction};

use crate::server::utilities::config_parse::Config;

// Import macros
use super::macros::{
    return_client_error, return_ok, return_ok_with_value, return_server_error, send_data_request,
};

/// Read parameters from request then execute them
///
/// # Parameters
/// - `request`: Request that has been read from socket
/// - `data_sender`: Sender that send data to database thread
/// - `config`: Application's configuration
pub async fn parse_request(
    request: Vec<u8>,
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
    config: Arc<RwLock<Config>>,
) -> Result<Vec<u8>, String> {
    // List all valid actions it will be matched later
    let valid_commands = vec![
        "SET",
        "GET",
        "REMKEY",
        "REMPATH",
        "LIST",
        "TRIGGER",
        "GETHOOK",
        "SETHOOK",
        "REMHOOK",
        "LISTHOOKS",
        "SUSPEND",
        "RESUME",
        "EXEC",
        "PUSH",
        "POP",
    ];
    let request = match String::from_utf8(request) {
        Ok(req) => req,
        Err(e) => return Err(format!("failed to read request: {}", e)),
    };

    let mut command = String::new();
    let mut key = String::new();
    let mut value = String::new();
    let mut copy = 0;

    // Go through characters and parse the request
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

    tracing::debug!(
        "traced paramerets: Command: {}, Key: {}, Value: {}",
        command,
        key,
        value
    );

    // Execute what the request asked then return with a reponse
    Ok(handle_command(command, key, value, data_sender, config).await)
}

/// Requst has been parsed and this function executes what it is made
///
/// # Parameters
/// - `command`: Sction verb about what to do
/// - `value`: This is all other characters that after the first words
/// - `data_sender`: Sender that send data to database thread
/// - `config`: Application's configuration
async fn handle_command(
    command: String,
    key: String,
    value: String,
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
    config: Arc<RwLock<Config>>,
) -> Vec<u8> {
    // Key is required for all request
    if key.is_empty() {
        tracing::trace!("key is missing");
        return_client_error!("Key is missing");
    }

    match command.as_str() {
        //
        // Create or update record
        //
        "SET" => {
            // SET without value is an error
            if value.is_empty() {
                tracing::debug!("no value specified for SET action");
                return_client_error!("Value is missing")
            }

            // Handle SET request
            let (tx, mut rx) = channel(10);
            let set_action = DatabaseAction::Set(tx, key, value);
            send_data_request!(set_action, data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(_) => return_ok!(),
                    Err(e) => return_client_error!(e),
                },
                None => return_server_error!("failed to receive message from database"),
            }
        }
        //
        // Get value of specific key
        //
        "GET" => {
            // Handle GET request
            let (tx, mut rx) = channel(10);
            let get_action = DatabaseAction::Get(tx, key);
            send_data_request!(get_action, data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(value) => match value {
                        ValueType::RecordPointer(data) => return_ok_with_value!(data),
                        _ => return_server_error!("Pointer must be Record but it was Table"),
                    },
                    Err(e) => return_client_error!(e),
                },
                None => return_server_error!("failed to receive message from database"),
            }
        }
        //
        // List all keys under a specified prefix
        //
        "LIST" => {
            // Handle LIST request
            let (tx, mut rx) = channel(10);
            let list_action = DatabaseAction::ListKeys(
                tx,
                key,
                onlyati_datastore::datastore::enums::ListType::All,
            );
            send_data_request!(list_action, data_sender);

            match rx.recv().await {
                Some(response) => match response {
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
                None => return_server_error!("failed to receive message from database"),
            }
        }
        //
        // Send a trigger, key-value pair is not saved but send to hook manager
        //
        "TRIGGER" => {
            // TRIGGER without value is an error
            if value.is_empty() {
                tracing::debug!("no value specified for SET action");
                return_client_error!("Value is missing")
            }

            // Handle TRIGGER request
            let (tx, mut rx) = channel(10);
            let trigger_action = DatabaseAction::Trigger(tx, key, value);
            send_data_request!(trigger_action, data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(_) => return_ok!(),
                    Err(e) => return_client_error!(e),
                },
                None => return_server_error!("failed to receive message from database"),
            }
        }
        //
        // Remove existing key
        //
        "REMKEY" => {
            // Handle REMKEY request
            let (tx, mut rx) = channel(10);
            let rem_action = DatabaseAction::DeleteKey(tx, key);
            send_data_request!(rem_action, data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(_) => return_ok!(),
                    Err(e) => return_client_error!(e),
                },
                None => return_server_error!("failed to receive message from database"),
            }
        }
        //
        // Remove existing path
        //
        "REMPATH" => {
            // Handle REMPATH request
            let (tx, mut rx) = channel(10);
            let rem_action = DatabaseAction::DeleteTable(tx, key);
            send_data_request!(rem_action, data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(_) => return_ok!(),
                    Err(e) => return_client_error!(e),
                },
                None => return_server_error!("failed to receive message from database"),
            }
        }
        //
        // Create new hook
        //
        "SETHOOK" => {
            // Add new hook
            if value.is_empty() {
                tracing::debug!("no link specified for SETHOOK action");
                return_client_error!("Link is missing")
            }
            let prefix = key;
            let link = value;

            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::HookSet(tx, prefix, link);
            send_data_request!(action, data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(_) => return_ok!(),
                    Err(e) => return_client_error!(e),
                },
                None => return_server_error!("failed to receive message from database"),
            }
        }
        //
        // Check that hook exist
        //
        "GETHOOK" => {
            // Get links for a hook
            let prefix = key;
            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::HookGet(tx, prefix);
            send_data_request!(action, data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok((_prefix, links)) => {
                        let mut response = String::new();
                        for link in links {
                            response += &link[..];
                            response += "\n";
                        }
                        return_ok_with_value!(response);
                    }
                    Err(e) => return_client_error!(e),
                },
                None => return_server_error!("failed to receive message from database"),
            }
        }
        //
        // Remove existing hook
        //
        "REMHOOK" => {
            // Delete an existing hook
            if value.is_empty() {
                tracing::debug!("no link specified for SETHOOK action");
                return_client_error!("Link is missing")
            }
            let prefix = key;
            let link = value;

            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::HookRemove(tx, prefix, link);
            send_data_request!(action, data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(_) => return_ok!(),
                    Err(e) => return_client_error!(e),
                },
                None => return_server_error!("failed to receive message from database"),
            }
        }
        //
        // List all defined hook under a specified prefix
        //
        "LISTHOOKS" => {
            // List hooks based on a prefix
            let prefix = key;
            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::HookList(tx, prefix);
            send_data_request!(action, data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(hooks) => {
                        let mut response = String::new();
                        for (prefix, links) in hooks {
                            response += format!("{} {:?}\n", prefix, links).as_str();
                        }
                        return_ok_with_value!(response);
                    }
                    Err(e) => return_client_error!(e),
                },
                None => return_server_error!("failed to receive message from database"),
            }
        }
        //
        // Suepend database logging
        //
        "SUSPEND" => {
            // Resume log action
            if key != "LOG" {
                return_client_error!("Invalid command, you may wanted to write: SUSPEND LOG");
            }

            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::SuspendLog(tx);
            send_data_request!(action, data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(_) => return_ok!(),
                    Err(e) => return_client_error!(e),
                },
                None => return_server_error!("failed to receive message from database"),
            }
        }
        //
        // Resume database logging
        //
        "RESUME" => {
            // Suspend log action
            if key != "LOG" {
                return_client_error!("Invalid command, you may wanted to write: SUSPEND LOG");
            }

            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::ResumeLog(tx);
            send_data_request!(action, data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(_) => return_ok!(),
                    Err(e) => return_client_error!(e),
                },
                None => return_server_error!("failed to receive message from database"),
            }
        }
        //
        // Execute lua script
        //
        "EXEC" => {
            // Execute lua script and save its output if needed
            // EXEC_SET <key> <script> <set-or-trigger> <value>
            if value.is_empty() {
                tracing::debug!("only action is specified but rest should be needed");
                return_client_error!("Script name, type and value are missing")
            }

            let mut c1st_space: usize = 0;
            let mut c2nd_space: usize = 0;
            let mut index: usize = 0;

            for c in value.chars() {
                if c == ' ' && c1st_space == 0 {
                    c1st_space = index;
                    index += 1;
                    continue;
                } else if c == ' ' && c2nd_space == 0 {
                    c2nd_space = index;
                    break;
                }

                index += 1;
            }

            if c2nd_space == 0 {
                return_client_error!("Invalid command, type is missing");
            }

            if c1st_space == 0 {
                return_client_error!("Invalid command, script is missing");
            }

            tracing::debug!("breakpoints for split value: {} {}", c1st_space, c2nd_space);

            let script = &value[..c1st_space].to_string();
            let save = &value[c1st_space + 1..c2nd_space].to_string();
            let real_value = &value[c2nd_space + 1..].to_string();

            tracing::debug!(
                "execute '{}' script for '{}' key as '{}'",
                script,
                key,
                save
            );
            tracing::debug!("[{}]", real_value);

            // Get the old value of exists
            let (tx, mut rx) = channel(10);
            let get_action = DatabaseAction::Get(tx, key.clone());

            send_data_request!(get_action, data_sender);

            let old_pair = match rx.recv().await {
                Some(response) => match response {
                    Ok(value) => match value {
                        ValueType::RecordPointer(data) => Some((key.clone(), data)),
                        _ => return_server_error!("Pointer must be Record but it was Table"),
                    },
                    Err(_) => None,
                },
                None => return_server_error!("failed to receive message from database"),
            };

            // Get config
            let config = {
                let config = config.read().await;
                match &config.scripts {
                    Some(scr) => match scr.execs.contains(script) {
                        true => scr.clone(),
                        false => return_client_error!("requested script is not defined"),
                    },
                    None => return_client_error!("requested script is not defined"),
                }
            };

            let new_pair = (key.clone(), real_value.trim().to_string());

            // Call lua utility
            let modified_pair = match crate::server::utilities::lua::run(
                config,
                old_pair,
                new_pair,
                script.clone(),
                None,
            )
            .await
            {
                Ok(modified_pair) => modified_pair,
                Err(e) => return_server_error!(format!("error during script exection: {}", e)),
            };

            // Make a SET action for the modified pair
            if save == "SET" {
                if modified_pair.1.is_empty() {
                    let (tx, mut rx) = channel(10);

                    let action = DatabaseAction::DeleteKey(tx, modified_pair.0);
                    send_data_request!(action, data_sender);

                    match rx.recv().await {
                        Some(response) => match response {
                            Ok(_) => return_ok!(),
                            Err(e) => return_client_error!(e.to_string()),
                        },
                        None => return_server_error!("failed to receive message from database"),
                    }
                } else {
                    let (tx, mut rx) = channel(10);
                    let action = DatabaseAction::Set(tx, modified_pair.0, modified_pair.1);
                    send_data_request!(action, data_sender);

                    match rx.recv().await {
                        Some(response) => match response {
                            Ok(_) => return_ok!(),
                            Err(e) => return_client_error!(e.to_string()),
                        },
                        None => return_server_error!("failed to receive message from database"),
                    }
                }
            }
            // Or a TRIGGER if this was requested
            else if save == "TRIGGER" {
                if !modified_pair.1.is_empty() {
                    let (tx, mut rx) = channel(10);
                    let action = DatabaseAction::Trigger(tx, modified_pair.0, modified_pair.1);
                    send_data_request!(action, data_sender);

                    match rx.recv().await {
                        Some(response) => match response {
                            Ok(_) => return_ok!(),
                            Err(e) => return_client_error!(e.to_string()),
                        },
                        None => return_server_error!("failed to receive message from database"),
                    }
                } else {
                    return_client_error!("After script was run, the new value is empty");
                }
            } else {
                return_client_error!("Type can be either SET or TRIGGER");
            }
        }
        //
        // Push new item into a queue
        //
        "PUSH" => {
            // SET without value is an error
            if value.is_empty() {
                tracing::debug!("no value specified for SET action");
                return_client_error!("Value is missing")
            }

            // Handle SET request
            let (tx, mut rx) = channel(10);
            let set_action = DatabaseAction::Push(tx, key, value);
            send_data_request!(set_action, data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(_) => return_ok!(),
                    Err(e) => return_client_error!(e),
                },
                None => return_server_error!("failed to receive message from database"),
            }
        }
        //
        // Get an item from a queue
        //
        "POP" => {
            // Handle GET request
            let (tx, mut rx) = channel(10);
            let get_action = DatabaseAction::Pop(tx, key);
            send_data_request!(get_action, data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(value) => match value {
                        ValueType::RecordPointer(data) => return_ok_with_value!(data),
                        _ => return_server_error!("Pointer must be Record but it was Table"),
                    },
                    Err(e) => return_client_error!(e),
                },
                None => return_server_error!("failed to receive message from database"),
            }
        }
        _ => unreachable!(),
    }
}

/// Run Classic interface
///
/// # Parameters
/// - `request`: Request that has been read from socket
/// - `data_sender`: Sender that send data to database thread
/// - `config`: Application's configuration
pub async fn run_async(
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
    address: String,
    config: Arc<RwLock<Config>>,
) {
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
        let config = config.clone();
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
                    }
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
            let response = match parse_request(request, data_sender, config).await {
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
            let _ = socket.0.flush().await;
        });
    }
}
