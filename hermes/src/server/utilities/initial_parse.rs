// External depencies
use serde::Deserialize;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

// Internal depencies
use onlyati_datastore::datastore::{enums::DatabaseAction, utilities};

/// Represent a record in initial toml file
#[derive(Deserialize)]
pub struct Record {
    key: String,
    value: String,
}

/// Represent a hook in initial toml file
#[derive(Deserialize)]
pub struct Hook {
    prefix: String,
    links: Vec<String>,
}

/// Represent a list in initial toml file
#[derive(Deserialize)]
pub struct InitialData {
    record: Vec<Record>,
    hook: Vec<Hook>,
}

/// Parse initial file
pub fn parse_initial_file(
    path: &String,
    data_sender: &Sender<DatabaseAction>,
) -> Result<(), String> {
    let file_content = super::get_file_content(path)?;

    // Read initial file
    let config: InitialData = match toml::from_str(&file_content[..]) {
        Ok(data) => data,
        Err(e) => return Err(format!("Failed to parse initial file: {}", e)),
    };

    // Upload hooks
    for hook in config.hook {
        tracing::debug!("write hook with '{}' to the database", hook.prefix);
        for link in &hook.links {
            let (tx, rx) = utilities::get_channel_for_hook_set();
            let action = DatabaseAction::HookSet(tx, hook.prefix.clone(), link.clone());

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

    // Upload records
    for pair in config.record {
        tracing::debug!("write pair with '{}' to the database", pair.key);
        let (tx, rx) = channel();
        let action = DatabaseAction::Set(tx, pair.key, pair.value);

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

    return Ok(());
}
