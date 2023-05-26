// External depencies
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::Sender;

// Internal depencies
use onlyati_datastore::datastore::{enums::DatabaseAction, utilities};

/// Parse the input file and upload onto database before anything would happen
pub fn parse_input_data(
    setting_name: &str,
    config: &HashMap<String, String>,
    data_sender: &Sender<DatabaseAction>,
) -> Result<(), String> {
    if let Some(path) = config.get(setting_name) {
        let file_content = get_file_content(path)?;

        for line in file_content.lines() {
            // Parse the words
            let (key, value) = match separate_words(line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            // Then upload onto database
            let (tx, rx) = utilities::get_channel_for_set();
            let action = DatabaseAction::Set(tx, key, value);

            if let Err(e) = data_sender.send(action) {
                return Err(format!("Error: {}", e));
            }

            match rx.recv() {
                Ok(response) => match response {
                    Err(e) => return Err(format!("Error: {}", e)),
                    _ => (),
                },
                Err(e) => return Err(format!("Error: {}", e)),
            }
        }
    }

    return Ok(());
}

/// Parse input hook file and upload onto database
pub fn parse_input_hook(setting_name: &str, config: &HashMap<String, String>, data_sender: &Sender<DatabaseAction>) -> Result<(), String> {
    if let Some(path) = config.get(setting_name) {
        let file_content = get_file_content(path)?;

        for line in file_content.lines() {
            // Parse the words
            let (prefix, link) = match separate_words(line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            // Then upload onto database
            let (tx, rx) = utilities::get_channel_for_hook_set();
            let action = DatabaseAction::HookSet(tx, prefix, link);

            if let Err(e) = data_sender.send(action) {
                return Err(format!("Error: {}", e));
            }

            match rx.recv() {
                Ok(response) => match response {
                    Err(e) => return Err(format!("Error: {}", e)),
                    _ => (),
                },
                Err(e) => return Err(format!("Error: {}", e)),
            }
        }

        // List defined hooks
        let (tx, rx) = utilities::get_channel_for_hook_list();
        let action = DatabaseAction::HookList(tx, "/root".to_string());
        if let Err(e) = data_sender.send(action) {
            return Err(format!("Error: {}", e));
        }

        match rx.recv() {
            Ok(response) => match response {
                Err(e) => return Err(format!("Error: {}", e)),
                Ok(list) => {
                    tracing::info!("Defined hooks at startup: ");
                    for (prefix, links) in list {
                        tracing::info!("- {}", prefix);
                        for link in links {
                            tracing::info!("  - {}", link);
                        }
                    }
                },
            },
            Err(e) => return Err(format!("Error: {}", e)),
        }
    }

    return Ok(());
}

/// Read a file content
fn get_file_content(path: &String) -> Result<String, String> {
    let path = Path::new(path);
    match path.exists() {
        true => match std::fs::read_to_string(path) {
            Ok(content) => return Ok(content),
            Err(e) => {
                return Err(format!(
                    "File '{}' could not been read: {}",
                    path.display(),
                    e
                ))
            }
        },
        false => return Err(format!("File '{}' does not exist", path.display())),
    }
}

/// Cut the first word from the line and return with a split value
fn separate_words(line: &str) -> Result<(String, String), ()> {
    if line.is_empty() {
        return Err(());
    }

    if &line[0..1] == " " {
        return Err(());
    }

    let mut end_of_key: usize = 0;
    let mut start_of_value: usize = 0;
    let mut index: usize = 0;
    for char in line.chars() {
        if char == ' ' && end_of_key == 0 {
            end_of_key = index;
            continue;
        }

        if char != ' ' && end_of_key != 0 {
            start_of_value = index + 1;
            break;
        }

        index += 1;
    }

    if start_of_value == 0 || end_of_key == 0 {
        return Err(());
    }

    // Allocate strings
    let key = String::from(&line[0..end_of_key]);
    let value = String::from(&line[start_of_value..]);

    return Ok((key, value));
}
