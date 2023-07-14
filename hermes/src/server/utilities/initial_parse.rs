// External depencies
use serde::Deserialize;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Sender;

// Internal depencies
use onlyati_datastore::datastore::enums::DatabaseAction;

/// Represent a record in initial toml file
#[derive(Deserialize)]
pub struct Record {
    key: String,
    value: String,
    r#override: Option<bool>,
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

/// Parse initial file and upload Hermes with pre-defined pairs.
/// First hooks are defined, then pairs, so if any pair would trigger
/// a hook, then it will during Hermes startup.
///
/// # Parameters
/// - `path`: Path to initial file
/// - `data_sender`: Sender that sends data to core database.
///
/// # Return
///
/// With Ok if no issue, else with error text.
pub async fn parse_initial_file(
    path: &String,
    data_sender: &Sender<DatabaseAction>,
) -> Result<(), String> {
    let file_content = super::get_file_content(path)?;

    // Read initial file
    let mut config: InitialData = match toml::from_str(&file_content[..]) {
        Ok(data) => data,
        Err(e) => return Err(format!("Failed to parse initial file: {}", e)),
    };

    // Upload hooks
    if let Some(hooks) = &mut config.hook {
        for hook in hooks {
            // Check that hook already exists
            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::HookGet(tx, hook.prefix.clone());

            if let Err(e) = data_sender.send(action).await {
                return Err(format!("Error: {}", e));
            }

            match rx.recv().await {
                Some(response) => {
                    if let Ok(restored) = response {
                        let mut to_be_removed = Vec::new();
                        let hook_links_len = hook.links.len();

                        for i in 0..hook_links_len {
                            if restored.1.contains(&hook.links[i]) {
                                to_be_removed.push(i);
                            }
                        }

                        for index in to_be_removed.iter().rev() {
                            tracing::debug!(
                                "hook {}@{} already exists in Hermes, its value remain",
                                hook.prefix,
                                hook.links[*index]
                            );
                            hook.links.remove(*index);
                        }

                        if hook.links.is_empty() {
                            continue;
                        }
                    }
                }
                None => return Err("failed to get record".to_string()),
            }

            // Set hook into Hermes
            tracing::debug!("write hook with '{}' to the database", hook.prefix);
            for link in &hook.links {
                let (tx, mut rx) = channel(10);
                let action = DatabaseAction::HookSet(tx, hook.prefix.clone(), link.clone());

                if let Err(e) = data_sender.send(action).await {
                    return Err(format!("Error: {}", e));
                }

                match rx.recv().await {
                    Some(response) => {
                        if let Err(e) = response {
                            return Err(format!("Error: {}", e));
                        }
                    }
                    None => return Err("failed to write hook".to_string()),
                }
            }
        }
    }

    // Upload records
    if let Some(records) = &config.record {
        for pair in records {
            // Check record already exist, if not don't override
            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::Get(tx, pair.key.clone());

            if let Err(e) = data_sender.send(action).await {
                return Err(format!("Error: {}", e));
            }

            match rx.recv().await {
                Some(response) => {
                    if response.is_ok() {
                        if pair.r#override != Some(true) {
                            tracing::debug!(
                                "key {} already exists in Hermes, its value remain",
                                pair.key
                            );
                            continue;
                        } else {
                            tracing::debug!(
                                "key {} already exists in Hermes, but it has override flag",
                                pair.key
                            );
                        }
                    }
                }
                None => return Err("failed to get record".to_string()),
            }

            // Write record into Hermes
            tracing::debug!("write pair with '{}' to the database", pair.key);
            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::Set(tx, pair.key.clone(), pair.value.clone());

            if let Err(e) = data_sender.send(action).await {
                return Err(format!("Error: {}", e));
            }

            match rx.recv().await {
                Some(response) => {
                    if let Err(e) = response {
                        return Err(format!("Error: {}", e));
                    }
                }
                None => return Err("failed to write record".to_string()),
            }
        }
    }

    Ok(())
}
