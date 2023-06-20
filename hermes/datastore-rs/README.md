# OnlyAti.Datastore
 
This a key-value in-memory database. This package has ability to:
- Run Datastore to work with key-value pairs (string and queue value types)
- Run Hook Managaer that are triggerd if specific keys has been update
- Run Logger that logging the actions on async way

There is an application that wrap it with extra functions for usage: [Olympus@Hermes](https://git.thinkaboutit.tech/PublicProjects/olympus-hermes)

## Sample code to use the built-in thread server

There is a provided function that creates a thread, initialize database then return with a `std::sync::mpsc::Sender` so other thread can send request.
This a simple method to initialize this database, communication can be done by using channels. Hook manager can also be initailized on this way of option is provided.

```rust
use onlyati_datastore::datastore::{
    enums::{error::ErrorKind, DatabaseAction, pair::ValueType},
    utilities::{start_datastore},
};
use onlyati_datastore::hook::utilities::start_hook_manager;
use onlyati_datastore::logger::utilities::start_logger;
use std::sync::mpsc::channel;

let (hook_sender, _) = start_hook_manager();
let (logger_sender, _) = start_logger(&"/tmp/tmp-datastore-log.txt".to_string());

// Start a new database with active hook manager
let (sender, _) = start_datastore("root".to_string(), Some(hook_sender.clone()), Some(logger_sender));

// Send a POST request to specified address when records updated within /root/status
let (tx, rx) = channel();
let action = DatabaseAction::HookSet(tx, "/root/network".to_string(), "http://127.0.0.1:3031".to_string());
sender.send(action).expect("Failed to send hook request");

rx.recv().expect("Failed to received response").expect("Bad request");

// Add a new pair
let (tx, rx) = channel();
let set_action = DatabaseAction::Set(tx, "/root/network/server1".to_string(), "ok".to_string());

sender.send(set_action).expect("Failed to send the request");
rx.recv().unwrap().unwrap();

// Get the pair
let (tx, rx) = channel();
let get_action = DatabaseAction::Get(tx, "/root/network/server1".to_string());

sender.send(get_action).expect("Failed to send the get request");
let data = rx.recv().expect("Failed to receive message").expect("Failed to get data");
assert_eq!(ValueType::RecordPointer("ok".to_string()), data);
```

## Sample code to run without built-in thread

There is a provided function that created a thread, initialize database then return with a `std::sync::mpsc::Sender` so other thread can send request.
But it is also possible to use it as it is called directly if the application does not prefer the method mentioned earlier.

```rust
use onlyati_datastore::controller::Database;
use onlyati_datastore::enums::{KeyType, ValueType, ListType};

// Logger and/or HookManager can be added with subscribe function
let mut db = onlyati_datastore::controller::Database::new("root".to_string()).unwrap();

let list: Vec<(KeyType, ValueType)> = vec![
    (KeyType::Record("/root/status/sub1".to_string()), ValueType::RecordPointer("OK".to_string())),
    (KeyType::Record("/root/status/sub2".to_string()), ValueType::RecordPointer("NOK".to_string())),
    (KeyType::Record("/root/network/dns".to_string()), ValueType::RecordPointer("OK".to_string())),
    (KeyType::Record("/root/network/www".to_string()), ValueType::RecordPointer("NOK".to_string())),
];

for (key, value) in list {
    db.insert(key, value).expect("Failed to insert");
}

let full_list = db.list_keys(KeyType::Record("/root".to_string()), ListType::All).expect("Failed to get all keys");
assert_eq!(true, full_list.len() == 4);
```

For more samples check `src/tests` direcotry.
