//! Main component

use std::{collections::VecDeque, sync::mpsc::Sender};

pub mod enums;
pub mod types;
pub mod utilities;

use crate::{hook::enums::HookManagerAction, logger::enums::LoggerAction};

use self::{
    enums::{error::ErrorKind, pair::KeyType, pair::ValueType, ListType},
    types::Table,
};

/// Database struct
pub struct Database {
    /// Name of database
    name: String,

    /// Pointer to the root table
    root: Table,

    /// Sender to HookManager
    hook_sender: Option<Sender<HookManagerAction>>,

    /// Logger function
    logger_sender: Option<Sender<LoggerAction>>,
}

impl Database {
    /// Create new database and return with the struct.
    ///
    /// # Arguments
    /// 1. `root_name` - Name of database
    ///
    /// # Examples
    /// ```
    /// let db = onlyati_datastore::datastore::Database::new("root".to_string()).unwrap();
    /// ```
    pub fn new(root_name: String) -> Result<Self, ErrorKind> {
        tracing::trace!(
            "try to allocate new database with '{}' root table",
            root_name
        );
        if root_name.contains('/') {
            return Err(ErrorKind::InvalidRoot(
                "Root name cannot contains '/' character".to_string(),
            ));
        }

        tracing::trace!("root table is allocated");
        Ok(Self {
            name: root_name,
            root: Table::new(),
            hook_sender: None,
            logger_sender: None,
        })
    }

    /// Subscribe to HookManager
    ///
    /// # Arguments
    /// 1. `sender` - Sender to HookManager thread
    ///
    /// # Examples
    /// ```
    /// let (sender, _) = onlyati_datastore::hook::utilities::start_hook_manager();
    /// let mut db = onlyati_datastore::datastore::Database::new("root".to_string()).unwrap();
    /// db.subscribe_to_hook_manager(sender);
    /// ```
    pub fn subscribe_to_hook_manager(&mut self, sender: Sender<HookManagerAction>) {
        tracing::trace!("subscribe to hook manager");
        self.hook_sender = Some(sender);
    }

    /// Subscribe to Logger
    ///
    /// # Arguments
    /// 1. `sender` - Sender to Logger thread
    ///
    /// # Examples
    /// ```
    /// let (sender, _) = onlyati_datastore::logger::utilities::start_logger(&"/tmp/datastore-tmp.txt".to_string());
    /// let mut db = onlyati_datastore::datastore::Database::new("root".to_string()).unwrap();
    /// db.subscribe_to_logger(sender);
    /// ```
    pub fn subscribe_to_logger(&mut self, sender: Sender<LoggerAction>) {
        tracing::trace!("subscribe to logger");
        self.logger_sender = Some(sender);
    }

    /// Insert or update key into database. Return with nothing if the insert was successful. Else with an error code.
    ///
    /// # Arguments
    /// 1. `key` - Unique key for data
    /// 1. `value` - Value that is assigned for the key
    ///
    /// # Example
    ///
    /// ```
    /// use onlyati_datastore::datastore::Database;
    /// use onlyati_datastore::datastore::enums::pair::{KeyType, ValueType};
    ///
    /// let mut db = Database::new("root".to_string()).unwrap();
    ///
    /// let result = db.insert(KeyType::Record("/root/network/dns-stats".to_string()), ValueType::RecordPointer("ok".to_string()));
    /// ```
    pub fn insert(&mut self, key: KeyType, value: ValueType) -> Result<(), ErrorKind> {
        tracing::trace!("set request is performed for '{}'", key.get_key());

        let key_routes = utilities::internal::validate_key(key.get_key(), &self.name)?;

        let mut table = Box::new(&mut self.root);
        let last_route = key_routes[key_routes.len() - 1];
        let mut route_index: usize = 0;
        let mut current_route = key_routes[route_index].to_string();

        while last_route != current_route {
            let temp_key = KeyType::Table(current_route);
            table
                .entry(temp_key.clone())
                .or_insert(ValueType::TablePointer(Table::new()));

            *table = match table.get_mut(&temp_key) {
                Some(item) => match item {
                    ValueType::TablePointer(sub_table) => sub_table,
                    _ => {
                        tracing::error!("wow, this should not happen a table pointer should be here not a record pointer");
                        return Err(ErrorKind::InternalError(
                            "This should not have happen".to_string(),
                        ));
                    }
                },
                _ => {
                    tracing::error!("wow, this should not happen table must exist");
                    return Err(ErrorKind::InternalError(
                        "This should not have happen".to_string(),
                    ));
                }
            };

            route_index += 1;
            current_route = key_routes[route_index].to_string();
        }

        let record_key = KeyType::Record(last_route.to_string());
        table.insert(record_key, value.clone());
        tracing::trace!("set request is done for '{}'", key.get_key());

        if let Some(sender) = &self.hook_sender {
            tracing::trace!("send alert to hook manager about '{}' key", key.get_key());
            if let ValueType::RecordPointer(value) = &value {
                let action = HookManagerAction::Send(key.get_key().to_string(), value.to_string());

                sender
                    .send(action)
                    .unwrap_or_else(|e| tracing::error!("Error during send: {}", e));
            }
        }

        Ok(())
    }

    /// Push a value into a queue. Return with nothing if the insert was successful. Else with an error code.
    ///
    /// # Arguments
    /// 1. `key` - Unique key for data
    /// 1. `value` - Value that will be pushed to queue
    ///
    /// # Example
    /// ```
    /// use onlyati_datastore::datastore::Database;
    /// use onlyati_datastore::datastore::enums::pair::{KeyType, ValueType};
    ///
    /// let mut db = Database::new("root".to_string()).unwrap();
    ///
    /// let result = db.push(KeyType::Record("/root/ticket/open".to_string()), "SINC100".to_string()).expect("Failed to push");
    /// let result = db.push(KeyType::Record("/root/ticket/open".to_string()), "SINC101".to_string()).expect("Failed to push");
    /// ```
    pub fn push(&mut self, key: KeyType, value: String) -> Result<(), ErrorKind> {
        tracing::trace!("push request is performed for '{}'", key.get_key());
        let key = match key {
            KeyType::Record(key) => key,
            _ => {
                return Err(ErrorKind::InvalidKey(
                    "Parameter must be a Record type".to_string(),
                ));
            }
        };

        let key_routes = utilities::internal::validate_key(&key[..], &self.name)?;

        let mut table = Box::new(&mut self.root);
        let last_route = key_routes[key_routes.len() - 1];
        let mut route_index: usize = 0;
        let mut current_route = key_routes[route_index].to_string();

        while last_route != current_route {
            let temp_key = KeyType::Table(current_route);
            table
                .entry(temp_key.clone())
                .or_insert(ValueType::TablePointer(Table::new()));

            *table = match table.get_mut(&temp_key) {
                Some(item) => match item {
                    ValueType::TablePointer(sub_table) => sub_table,
                    _ => {
                        tracing::error!("wow, this should not happen a table pointer should be here not a record pointer");
                        return Err(ErrorKind::InternalError(
                            "This should not have happen".to_string(),
                        ));
                    }
                },
                _ => {
                    tracing::error!("wow, this should not happen table must exist");
                    return Err(ErrorKind::InternalError(
                        "This should not have happen".to_string(),
                    ));
                }
            };

            route_index += 1;
            current_route = key_routes[route_index].to_string();
        }

        match table.get_mut(&KeyType::Queue(last_route.to_string())) {
            Some(elem) => match elem {
                ValueType::QueuePointer(queue) => {
                    queue.push_back(value.clone());
                    tracing::trace!("push request is done for '{}'", key);

                    if let Some(sender) = &self.hook_sender {
                        tracing::trace!("send alert to hook manager about '{}' key", key);
                        let action = HookManagerAction::Send(key, value);

                        sender
                            .send(action)
                            .unwrap_or_else(|e| tracing::error!("Error during send: {}", e));
                    }
                }
                _ => {
                    tracing::trace!("queue '{}' does not exist", key);
                    return Err(ErrorKind::InvalidKey(
                        "Specified key does not exist".to_string(),
                    ));
                }
            },
            None => {
                let new_qeue = KeyType::Queue(last_route.to_string());
                let mut queue = VecDeque::new();
                queue.push_back(value);
                table.insert(new_qeue, ValueType::QueuePointer(queue));
            }
        }

        Ok(())
    }

    /// Send a trigger to HookManager, record is not created like at `insert` but it can trigger and send some hooks out
    ///
    /// # Arguments
    ///
    /// 1. `key` - Unique key for data
    /// 1. `value` - Value that is assigned for the key
    ///
    /// # Examples
    ///
    /// ```
    /// use onlyati_datastore::datastore::Database;
    /// use onlyati_datastore::datastore::enums::pair::{KeyType, ValueType};
    ///
    /// let mut db = Database::new("root".to_string()).unwrap();
    ///
    /// let result = db.trigger(KeyType::Record("/root/network/dns-stats".to_string()), ValueType::RecordPointer("ok".to_string()));
    /// ```
    pub fn trigger(&self, key: KeyType, value: ValueType) -> Result<(), ErrorKind> {
        match &self.hook_sender {
            Some(sender) => {
                tracing::trace!("send trigger to hook manager about '{}' key", key.get_key());
                if let ValueType::RecordPointer(value) = &value {
                    let action =
                        HookManagerAction::Send(key.get_key().to_string(), value.to_string());

                    sender
                        .send(action)
                        .unwrap_or_else(|e| tracing::error!("Error during send: {}", e));
                }
                Ok(())
            }
            None => Err(ErrorKind::InactiveHookManager),
        }
    }

    /// Get the value of a key and return with a copy of it. If not found return with error.
    ///
    /// # Arguments
    /// 1. `key` - Unique key that has to be found
    ///
    /// # Example
    ///
    /// ```
    /// use onlyati_datastore::datastore::Database;
    /// use onlyati_datastore::datastore::enums::pair::{KeyType, ValueType};
    ///
    /// let mut db = Database::new("root".to_string()).unwrap();
    ///
    /// db.insert(KeyType::Record("/root/status".to_string()), ValueType::RecordPointer("Having a great time".to_string())).expect("Failed to insert");
    /// let value = db.get(KeyType::Record("/root/status".to_string())).expect("Key not found");
    /// ```
    pub fn get(&self, key: KeyType) -> Result<ValueType, ErrorKind> {
        tracing::trace!("get request is performed for '{}'", key.get_key());
        let key = match key {
            KeyType::Record(key) => key,
            _ => {
                return Err(ErrorKind::InvalidKey(
                    "Parameter must be a Record type".to_string(),
                ));
            }
        };

        let key_routes = utilities::internal::validate_key(&key[..], &self.name)?;
        let table = match utilities::internal::find_table(
            &self.root,
            key_routes[..key_routes.len() - 1].to_vec(),
        ) {
            Some(table) => table,
            None => {
                tracing::trace!("key '{}' does not exist", key);
                return Err(ErrorKind::InvalidKey(
                    "Specified key does not exist".to_string(),
                ));
            }
        };

        let find_key = KeyType::Record(key_routes[key_routes.len() - 1].to_string());

        match table.get(&find_key) {
            Some(value) => {
                tracing::trace!("get request is done for '{}'", key);
                Ok(value.clone())
            }
            None => {
                tracing::trace!("key '{}' does not exist", key);
                Err(ErrorKind::InvalidKey(
                    "Specified key does not exist".to_string(),
                ))
            }
        }
    }

    /// Pop value from queue. If not found return with error.
    ///
    /// # Arguments
    /// 1. `key` - Unique key that has to be found
    ///
    /// # Example
    ///
    /// ```
    /// use onlyati_datastore::datastore::Database;
    /// use onlyati_datastore::datastore::enums::pair::{KeyType, ValueType};
    ///
    /// let mut db = Database::new("root".to_string()).unwrap();
    ///
    /// let result = db.push(KeyType::Record("/root/ticket/open".to_string()), "SINC100".to_string()).expect("Failed to push");
    /// let result = db.push(KeyType::Record("/root/ticket/open".to_string()), "SINC101".to_string()).expect("Failed to push");
    ///
    /// let ticket = db.pop(KeyType::Record("/root/ticket/open".to_string())).expect("Failed to pop");
    /// assert_eq!("SINC100".to_string(), ticket);
    ///
    /// let ticket = db.pop(KeyType::Record("/root/ticket/open".to_string())).expect("Failed to pop");
    /// assert_eq!("SINC101".to_string(), ticket);
    ///
    /// let ticket = db.pop(KeyType::Record("/root/ticket/open".to_string()));
    /// assert_eq!(true, ticket.is_err());
    /// ```
    pub fn pop(&mut self, key: KeyType) -> Result<String, ErrorKind> {
        tracing::trace!("get request is performed for '{}'", key.get_key());
        let key = match key {
            KeyType::Record(key) => key,
            _ => {
                return Err(ErrorKind::InvalidKey(
                    "Parameter must be a Record type".to_string(),
                ));
            }
        };

        let key_routes = utilities::internal::validate_key(&key[..], &self.name)?;
        let table = match utilities::internal::find_table_mut(
            &mut self.root,
            key_routes[..key_routes.len() - 1].to_vec(),
        ) {
            Some(table) => table,
            None => {
                tracing::trace!("key '{}' does not exist", key);
                return Err(ErrorKind::InvalidKey(
                    "Specified key does not exist".to_string(),
                ));
            }
        };

        let find_key = KeyType::Queue(key_routes[key_routes.len() - 1].to_string());

        match table.get_mut(&find_key) {
            Some(value) => {
                tracing::trace!("get request is done for '{}'", key);
                match value {
                    ValueType::QueuePointer(queue) => {
                        let ret_value = match queue.pop_front() {
                            Some(v) => v,
                            None => {
                                tracing::error!("queue was not cleanup before, try now");
                                table.remove(&find_key);
                                return Err(ErrorKind::InvalidKey(
                                    "Specified key does not exist".to_string(),
                                ));
                            }
                        };

                        if queue.is_empty() {
                            table.remove(&find_key);
                        }

                        Ok(ret_value)
                    }
                    _ => {
                        tracing::error!("this should not be happen, search was to a Queue but something else was found");
                        Err(ErrorKind::InvalidKey(
                            "Specified key does not exist".to_string(),
                        ))
                    }
                }
            }
            None => {
                tracing::trace!("key '{}' does not exist", key);
                Err(ErrorKind::InvalidKey(
                    "Specified key does not exist".to_string(),
                ))
            }
        }
    }

    /// List keys from a specific entry point and return with a key list. If failed return with error.
    ///
    /// # Arguments
    /// 1. `key_prefix` - Path where the keys has to be collected
    /// 1. `level` - Need all inner level (`ListType::All`) or just current level (`ListType::OneLevel`)
    ///
    /// # Example
    ///
    /// ```
    /// use onlyati_datastore::datastore::Database;
    /// use onlyati_datastore::datastore::enums::{pair::KeyType, pair::ValueType, ListType};
    ///
    /// let mut db = Database::new("root".to_string()).unwrap();
    ///
    /// db.insert(KeyType::Record("/root/status/sub1".to_string()), ValueType::RecordPointer("PING OK".to_string())).expect("Failed to insert");
    /// db.insert(KeyType::Record("/root/status/sub2".to_string()), ValueType::RecordPointer("PING NOK".to_string())).expect("Failed to insert");
    /// db.insert(KeyType::Record("/root/status/sub3".to_string()), ValueType::RecordPointer("PING OK".to_string())).expect("Failed to insert");
    /// let list = db.list_keys(KeyType::Record("/root/status".to_string()), ListType::All).expect("Key not found");
    ///
    /// println!("{:?}", list);
    /// ```
    pub fn list_keys(
        &mut self,
        key_prefix: KeyType,
        level: ListType,
    ) -> Result<Vec<KeyType>, ErrorKind> {
        tracing::trace!(
            "list keys request is performed for '{}'",
            key_prefix.get_key()
        );
        let key_prefix = match key_prefix {
            KeyType::Record(key) => key,
            _ => {
                return Err(ErrorKind::InvalidKey(
                    "Parameter must be a Record type".to_string(),
                ));
            }
        };

        // Find the base table
        let key_routes = utilities::internal::validate_key(&key_prefix[..], &self.name)?;
        let table = match utilities::internal::find_table(&self.root, key_routes) {
            Some(table) => table,
            None => {
                tracing::trace!("get request is failed due to no '{}' key exist", key_prefix);
                return Err(ErrorKind::InvalidKey(
                    "Specified route does not exist".to_string(),
                ));
            }
        };

        // Get the information
        let result = utilities::internal::display_tables(table, &key_prefix, &level)?;

        tracing::trace!("list keys request is done for '{}'", key_prefix);
        Ok(result)
    }

    /// Delete specific key, return with nothig if successful, else with error message.
    ///
    /// # Arguments
    /// 1. `key` - Unique key that has to be deleted
    ///
    /// # Example
    ///
    /// ```
    /// use onlyati_datastore::datastore::Database;
    /// use onlyati_datastore::datastore::enums::pair::{KeyType, ValueType};
    ///
    /// let mut db = Database::new("root".to_string()).unwrap();
    ///
    /// let key = KeyType::Record("/root/status".to_string());
    /// db.insert(key.clone(), ValueType::RecordPointer("Having a great time".to_string())).expect("Failed to insert");
    /// db.delete_key(key).expect("Could not delete the key");
    /// ```
    pub fn delete_key(&mut self, key: KeyType) -> Result<(), ErrorKind> {
        tracing::trace!("delete key request is performed for '{}'", key.get_key());
        if let KeyType::Table(_) = key {
            tracing::trace!("delete request is failed due to wrong key type");
            return Err(ErrorKind::InvalidKey(
                "Parameter must be a Record type".to_string(),
            ));
        }

        let key_routes = utilities::internal::validate_key(key.get_key(), &self.name)?;
        let table = match utilities::internal::find_table_mut(
            &mut self.root,
            key_routes[..key_routes.len() - 1].to_vec(),
        ) {
            Some(table) => table,
            None => {
                tracing::trace!(
                    "delete request is failed because no '{}' key exist",
                    key.get_key()
                );
                return Err(ErrorKind::InvalidKey(
                    "Specified key does not exist".to_string(),
                ));
            }
        };

        let delete_key = KeyType::Record(key_routes[key_routes.len() - 1].to_string());

        match table.remove(&delete_key) {
            Some(_) => {
                tracing::trace!("delete request is done for '{}'", key.get_key());
                Ok(())
            }
            None => {
                tracing::trace!(
                    "delete request is failed because no '{}' key exist",
                    key.get_key()
                );
                Err(ErrorKind::InvalidKey(
                    "Specified key does not exist".to_string(),
                ))
            }
        }
    }

    /// Drop the whole table. If successful return with nothing else with error message.
    ///
    /// # Arguments
    /// 1. `key` - Key that which table has to be deleted
    ///
    /// # Example
    ///
    /// ```
    /// use onlyati_datastore::datastore::Database;
    /// use onlyati_datastore::datastore::enums::{pair::KeyType, pair::ValueType, ListType};
    ///
    /// let mut db = Database::new("root".to_string()).unwrap();
    ///
    /// db.insert(KeyType::Record("/root/status/sub1".to_string()), ValueType::RecordPointer("PING OK".to_string())).expect("Failed to insert");
    /// db.insert(KeyType::Record("/root/status/sub2".to_string()), ValueType::RecordPointer("PING NOK".to_string())).expect("Failed to insert");
    /// db.insert(KeyType::Record("/root/status/sub3".to_string()), ValueType::RecordPointer("PING OK".to_string())).expect("Failed to insert");
    /// db.insert(KeyType::Record("/root/node_name".to_string()), ValueType::RecordPointer("vps01".to_string())).expect("Failed to insert");
    ///
    /// db.delete_table(KeyType::Table("/root/status".to_string())).expect("Failed to drop from status table");
    ///
    /// // Only "node_name" remain in the list
    /// let list = db.list_keys(KeyType::Record("/root".to_string()), ListType::All).expect("Key not found");
    /// println!("{:?}", list);
    /// ```
    pub fn delete_table(&mut self, key: KeyType) -> Result<(), ErrorKind> {
        tracing::trace!("delete table request is performed for '{}'", key.get_key());
        if let KeyType::Record(_) = key {
            tracing::trace!("delete table request is failed due to wrong key type is specified");
            return Err(ErrorKind::InvalidKey(
                "Parameter must be a Table type".to_string(),
            ));
        }

        let key_routes = utilities::internal::validate_key(key.get_key(), &self.name)?;
        let table = match utilities::internal::find_table_mut(
            &mut self.root,
            key_routes[..key_routes.len() - 1].to_vec(),
        ) {
            Some(table) => table,
            None => {
                tracing::trace!(
                    "delete table request is failed because no '{}' key exist",
                    key.get_key()
                );
                return Err(ErrorKind::InvalidKey(
                    "Specified key does not exist".to_string(),
                ));
            }
        };

        let delete_key = KeyType::Table(key_routes[key_routes.len() - 1].to_string());

        match table.remove(&delete_key) {
            Some(_) => {
                tracing::trace!("delete table request is performed for '{}'", key.get_key());
                Ok(())
            }
            None => {
                tracing::trace!(
                    "delete table request is failed because no '{}' key exist",
                    key.get_key()
                );
                Err(ErrorKind::InvalidKey(
                    "Specified key does not exist".to_string(),
                ))
            }
        }
    }
}
