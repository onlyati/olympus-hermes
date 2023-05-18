use std::collections::BTreeMap;
use std::sync::mpsc::{channel, Sender};

type Prefix = String;
type Key = String;
type Link = String;
type Value = String;
type Hooks = Vec<Link>;

pub struct HookManager {
    hooks: BTreeMap<Prefix, Hooks>,
}

impl HookManager {
    pub fn new() -> Self {
        return HookManager {
            hooks: BTreeMap::new(),
        };
    }

    pub fn add(&mut self, prefix: String, link: String) -> Result<(), HookManagerResponse> {
        match self.hooks.get_mut(&prefix) {
            Some(hooks) => match hooks.iter().position(|x| x == &link) {
                Some(_) => return Err(HookManagerResponse::Error("Already defined".to_string())),
                None => {
                    hooks.push(link);
                    return Ok(());
                }
            },
            None => {
                self.hooks.insert(prefix, vec![link]);
                return Ok(());
            }
        }
    }

    pub fn remove(&mut self, prefix: String, link: String) -> Result<(), HookManagerResponse> {
        match self.hooks.get_mut(&prefix) {
            Some(hooks) => match hooks.iter().position(|x| x == &link) {
                Some(index) => {
                    hooks.remove(index);
                    return Ok(());
                }
                None => return Err(HookManagerResponse::Error("Not found".to_string())),
            },
            None => return Err(HookManagerResponse::Error("Not found".to_string())),
        }
    }

    pub fn get(&self, prefix: &String) -> Option<Hooks> {
        match self.hooks.get(prefix) {
            Some(hooks) => return Some(hooks.clone()),
            None => return None,
        }
    }

    pub fn list(&self, key: &String) -> BTreeMap<Prefix, Hooks> {
        let selected_hooks: BTreeMap<Prefix, Hooks> = self
            .hooks
            .iter()
            .filter(|x| key.starts_with(x.0))
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect();
        return selected_hooks;
    }

    pub async fn execute_hooks(&self, key: &String, value: &String) {
        let client = reqwest::Client::new();
        let body = format!("{{ \"key\" : \"{}\", \"value\" : \"{}\" }}", key, value);

        for (prefix, links) in &self.hooks {
            if key.starts_with(prefix) {
                for link in links {
                    match client.post(link).body(body.clone()).send().await {
                        Err(e) => println!("Error: HTTP request with hook but: {}", e),
                        _ => (),
                    };
                }
            }
        }
    }
}

/// Input actions for HookManager
pub enum HookManagerAction {
    /// SET new hook
    Set(Sender<HookManagerResponse>, Prefix, Link),

    /// Remove existing hook
    Remove(Sender<HookManagerResponse>, Prefix, Link),

    /// Get that hook exist
    Get(Sender<HookManagerResponse>, Prefix),

    /// List hooks
    List(Sender<HookManagerResponse>, Prefix),

    /// Send data to defined hooks
    Send(Sender<HookManagerResponse>, Key, Value),
}

/// Possible answers for HookManager
pub enum HookManagerResponse {
    /// Empty good response
    Ok,

    /// Somthing wrong happened
    Error(String),

    /// Reponse for GET
    Hook(Prefix, Hooks),

    /// Response for LIST
    HookList(BTreeMap<Prefix, Hooks>),
}

pub fn start_manager() -> Sender<HookManagerAction> {
    let (tx, rx) = channel::<HookManagerAction>();
    let mut manager = HookManager::new();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to allocate runtime for HookManager");

    std::thread::spawn(move || {
        rt.block_on(async move {
            loop {
                match rx.recv() {
                    Ok(request) => match request {
                        HookManagerAction::Set(sender, prefix, target) => {
                            match manager.add(prefix, target) {
                                Ok(_) => sender
                                    .send(HookManagerResponse::Ok)
                                    .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                                Err(e) => sender
                                    .send(e)
                                    .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                            }
                        }
                        HookManagerAction::Remove(sender, prefix, target) => {
                            match manager.remove(prefix, target) {
                                Ok(_) => sender
                                    .send(HookManagerResponse::Ok)
                                    .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                                Err(e) => sender
                                    .send(e)
                                    .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                            }
                        }
                        HookManagerAction::Get(sender, prefix) => match manager.get(&prefix) {
                            Some(hooks) => sender
                                .send(HookManagerResponse::Hook(prefix, hooks))
                                .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                            None => sender
                                .send(HookManagerResponse::Error("Not found".to_string()))
                                .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                        },
                        HookManagerAction::List(sender, prefix) => {
                            sender
                                .send(HookManagerResponse::HookList(manager.list(&prefix)))
                                .unwrap_or_else(|e| eprintln!("Error during send: {}", e));
                        }
                        HookManagerAction::Send(sender, test_key, value) => {
                            manager.execute_hooks(&test_key, &value).await;
                        }
                    },
                    Err(e) => panic!("Hook manager failed: {}", e),
                }
            }
        });
    });

    return tx;
}
