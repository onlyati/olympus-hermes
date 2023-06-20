//! Main component

use std::collections::{BTreeMap, HashMap};

pub mod enums;
pub mod types;
pub mod utilities;

use enums::HookManagerResponse;
use types::{Hooks, Prefix};

/// HookManager main structure
/// 
/// # Examples
/// ```
/// use onlyati_datastore::hook::HookManager;
/// 
/// let mut manager = HookManager::new();
/// 
/// let mut manager = HookManager::new();
/// 
/// let result = manager.add("/root/status".to_string(), "http://127.0.0.1:3031".to_string());
/// assert_eq!(true, result.is_ok());
/// 
/// let result = manager.add("/root/status".to_string(), "http://127.0.0.1:3032".to_string());
/// assert_eq!(true, result.is_ok());
/// 
/// let result = manager.add("/root/arpa".to_string(), "http://127.0.0.1:3031".to_string());
/// assert_eq!(true, result.is_ok());
/// 
/// let result = manager.list(&"/root".to_string());
/// assert_eq!(2, result.len());
/// 
/// let result = manager.list(&"/root/stat".to_string());
/// assert_eq!(1, result.len());
/// 
/// let result = manager.list(&"/root/no_exist".to_string());
/// assert_eq!(0, result.len());
/// ```
#[derive(Clone)]
pub struct HookManager {
    /// List about hooks
    hooks: BTreeMap<Prefix, Hooks>,
    client: reqwest::Client,
}

impl HookManager {
    /// Allocate new HookManager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add new hook
    pub fn add(&mut self, prefix: String, link: String) -> Result<(), HookManagerResponse> {
        tracing::trace!("hook set request is performed for '{}' prefix with '{}' link", prefix, link);
        match self.hooks.get_mut(&prefix) {
            Some(hooks) => match hooks.iter().position(|x| x == &link) {
                Some(_) => {
                    tracing::trace!("hook set request is failed due to '{}' with '{}' link is already exist", prefix, link);
                    Err(HookManagerResponse::Error("Already defined".to_string()))
                },
                None => {
                    hooks.push(link);
                    tracing::trace!("hook set request is done for '{}'", prefix);
                    Ok(())
                }
            },
            None => {
                tracing::trace!("hook set request is done for '{}'", prefix);
                self.hooks.insert(prefix, vec![link]);
                Ok(())
            }
        }
    }

    /// Delete existing hook
    pub fn remove(&mut self, prefix: String, link: String) -> Result<(), HookManagerResponse> {
        tracing::trace!("hook set request is performed for '{}' prefix with '{}' link", prefix, link);
        match self.hooks.get_mut(&prefix) {
            Some(hooks) => {
                match hooks.iter().position(|x| x == &link) {
                    Some(index) => hooks.remove(index),
                    None => {
                        tracing::trace!("hook set request is failed because no '{}' link exist", link);
                        return Err(HookManagerResponse::Error("Not found".to_string()));
                    },
                };

                if hooks.is_empty() {
                    self.hooks.remove(&prefix);
                }

                tracing::trace!("hook set request is done for '{}' prefix with '{}' link", prefix, link);
                Ok(())
            },
            None => {
                tracing::trace!("hook set request is failed because no '{}' hook found", prefix);
                Err(HookManagerResponse::Error("Not found".to_string()))
            },
        }
    }

    /// Check that hook exist
    pub fn get(&self, prefix: &String) -> Option<Hooks> {
        tracing::trace!("hook get request is performed for '{}' prefix", prefix);
        match self.hooks.get(prefix) {
            Some(hooks) => {
                tracing::trace!("hook get request is done for '{}' prefix", prefix);
                Some(hooks.clone())
            },
            None => {
                tracing::trace!("hook get request is failed due to no '{}' prefix exist", prefix);
                None
            },
        }
    }

    /// List hooks for specified paths
    pub fn list(&self, key: &String) -> BTreeMap<Prefix, Hooks> {
        tracing::trace!("hook list request is performed for '{}' prefix", key);
        let selected_hooks: BTreeMap<Prefix, Hooks> = self
            .hooks
            .iter()
            .filter(|x| x.0.starts_with(key))
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect();
        tracing::trace!("hook list request is done and found {} record", selected_hooks.len());
        selected_hooks
    }

    /// Pass a key and send POST request if key match with any defined prefix
    /// 
    /// # Examples
    /// ```
    /// use onlyati_datastore::hook::HookManager;
    /// 
    /// let mut manager = HookManager::new();
    /// 
    /// let mut manager = HookManager::new();
    /// 
    /// // Normaly you have to specify address where the HTTP POST request can be sent
    /// let result = manager.add("/root/status".to_string(), "http://127.0.0.1:3031".to_string());
    /// assert_eq!(true, result.is_ok());
    /// 
    /// let result = manager.add("/root/status".to_string(), "http://127.0.0.1:3032".to_string());
    /// assert_eq!(true, result.is_ok());
    /// 
    /// let rt = tokio::runtime::Builder::new_current_thread()
    ///     .enable_all()
    ///     .build()
    ///     .unwrap();
    /// rt.block_on(async move {
    ///     let counter = manager.execute_hooks(&"/root/status/dns1".to_string(), &"okay".to_string()).await;
    ///     assert_eq!(Some(2), counter);
    /// 
    ///     let counter = manager.execute_hooks(&"/root/no_exist".to_string(), &"okay".to_string()).await;
    ///     assert_eq!(None, counter);
    /// });
    /// 
    /// ```
    pub async fn execute_hooks(&self, key: &String, value: &String) -> Option<i32> {
        let mut body = HashMap::new();
        body.insert("key", key);
        body.insert("value", value);
        tracing::debug!("check hooks for {}", key);

        let mut counter = 0;

        for (prefix, links) in &self.hooks {
            if key.starts_with(prefix) {
                for link in links {
                    tracing::trace!("send POST request to '{}' link", link);
                    counter += 1;
                    match self.client.post(link).json(&body).send().await {
                        Err(e) => tracing::error!("Error: HTTP request with hook but: {}", e),
                        Ok(resp) => tracing::trace!("{:?}", resp),
                    };
                }
            }
        }

        tracing::trace!("sent {} request for '{}' key", counter, key);

        match counter {
            0 => None,
            i => Some(i),
        }
    }
}

/// Default implementation of HookManager
impl Default for HookManager {
    fn default() -> Self {
        Self {
            hooks: BTreeMap::new(),
            client: reqwest::Client::new(),
        }
    }
}
