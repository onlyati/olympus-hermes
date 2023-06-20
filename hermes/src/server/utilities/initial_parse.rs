// External depencies
use serde::Deserialize;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

// Internal depencies
use onlyati_datastore::datastore::{enums::DatabaseAction};

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
    record: Option<Vec<Record>>,
    hook: Option<Vec<Hook>>,
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
    if let Some(hooks) = &config.hook {
        for hook in hooks {
            tracing::debug!("write hook with '{}' to the database", hook.prefix);
            for link in &hook.links {
                let (tx, rx) = channel();
                let action = DatabaseAction::HookSet(tx, hook.prefix.clone(), link.clone());

                if let Err(e) = data_sender.send(action) {
                    return Err(format!("Error: {}", e));
                }

                match rx.recv() {
                    Ok(response) => {
                        if let Err(e) = response {
                            return Err(format!("Error: {}", e));
                        }
                    }
                    Err(e) => return Err(format!("Error: {}", e)),
                }
            }
        }
    }

    // Upload records
    if let Some(records) = &config.record {
        for pair in records {
            tracing::debug!("write pair with '{}' to the database", pair.key);
            let (tx, rx) = channel();
            let action = DatabaseAction::Set(tx, pair.key.clone(), pair.value.clone());

            if let Err(e) = data_sender.send(action) {
                return Err(format!("Error: {}", e));
            }

            match rx.recv() {
                Ok(response) => {
                    if let Err(e) = response {
                        return Err(format!("Error: {}", e));
                    }
                }
                Err(e) => return Err(format!("Error: {}", e)),
            }
        }
    }

    Ok(())
}
