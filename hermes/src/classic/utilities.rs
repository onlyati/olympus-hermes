use onlyati_datastore::{
    enums::{DatabaseAction, ValueType},
    utilities,
};
use std::sync::{mpsc::Sender, Arc, Mutex};

use super::macros::{
    return_client_error, return_ok, return_ok_with_value, return_server_error, send_data_request,
};

/// Read parameters from request then execute them
pub fn parse_request(
    request: Vec<u8>,
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
) -> Result<Vec<u8>, String> {
    let valid_commands = vec![
        "SET",
        "GET",
        "REMKEY",
        "REMPATH",
        "LIST",
    ];
    let request = match String::from_utf8(request) {
        Ok(req) => req,
        Err(e) => return Err(format!("Failed to read request: {}", e)),
    };

    let mut command = String::new();
    let mut key = String::new();
    let mut value = String::new();
    let mut copy = 0;

    for byte in request.chars() {
        if copy == 0 {
            if byte == ' ' {
                if !valid_commands.contains(&command.as_str()) {
                    // If not valid command then don't check further
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

    return Ok(handle_command(command, key, value, data_sender));
}

fn handle_command(
    command: String,
    key: String,
    value: String,
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
) -> Vec<u8> {
    if key.is_empty() {
        return_client_error!("Key is missing");
    }

    match command.as_str() {
        "SET" => {
            if value.is_empty() {
                return_client_error!("Value is missing")
            }

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
            let (tx, rx) = utilities::get_channel_for_list();
            let list_action = DatabaseAction::ListKeys(tx, key, onlyati_datastore::enums::ListType::All);
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
                    },
                    Err(e) => return_client_error!(e),
                },
                Err(e) => return_server_error!(e),
            }
        }
        "REMKEY" => {
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
