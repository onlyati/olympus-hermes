#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use tokio::sync::mpsc::channel;

    use crate::{
        datastore::{
            enums::{error::ErrorKind, pair::KeyType, pair::ValueType, DatabaseAction, ListType},
            utilities::start_datastore,
            Database,
        },
        hook::HookManager,
    };

    #[test]
    fn list_test() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let db = Database::new("root".to_string());
            assert_eq!(true, db.is_ok());
            let mut db = db.unwrap();

            let list: Vec<(KeyType, ValueType)> = vec![
                (
                    KeyType::Record("/root/status/sub1".to_string()),
                    ValueType::RecordPointer("OK".to_string()),
                ),
                (
                    KeyType::Record("/root/status/sub2".to_string()),
                    ValueType::RecordPointer("NOK".to_string()),
                ),
                (
                    KeyType::Record("/root/network/dns".to_string()),
                    ValueType::RecordPointer("OK".to_string()),
                ),
                (
                    KeyType::Record("/root/network/www".to_string()),
                    ValueType::RecordPointer("NOK".to_string()),
                ),
            ];

            for (key, value) in list {
                db.insert(key, value).await.expect("Failed to insert");
            }

            let full_list = db
                .list_keys(KeyType::Record("/root".to_string()), ListType::All)
                .expect("Failed to get all keys");
            assert_eq!(true, full_list.len() == 4);
        });
    }

    #[test]
    fn server_test() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let (hook_sender, _) = crate::hook::utilities::start_hook_manager().await;
            let (sender, _) =
                start_datastore("root".to_string(), Some(hook_sender), None).await;

            // Add a new pair
            let (tx, mut rx) = channel(10);
            let set_action = DatabaseAction::Set(tx, "/root/network".to_string(), "ok".to_string());
            sender
                .send(set_action)
                .await
                .expect("Failed to send the request");
            rx.recv()
                .await
                .expect("Failed to send action")
                .expect("Failed to set value");

            let (tx, mut rx) = channel(10);
            let set_action =
                DatabaseAction::Set(tx, "/root/network".to_string(), "nok".to_string());
            sender
                .send(set_action)
                .await
                .expect("Failed to send the request");
            rx.recv()
                .await
                .expect("Failed to send action")
                .expect("Failed to set value");

            // Get the pair
            let (tx, mut rx) = channel(10);
            let get_action = DatabaseAction::Get(tx, "/root/network".to_string());

            sender
                .send(get_action)
                .await
                .expect("Failed to send the get request");
            let data = rx
                .recv()
                .await
                .expect("Failed to receive message")
                .expect("Failed to get data");
            assert_eq!(ValueType::RecordPointer("nok".to_string()), data);

            let (tx, mut rx) = channel(10);
            let trigger_action = DatabaseAction::Trigger(
                tx,
                "/root/new-test".to_string(),
                "placeholder".to_string(),
            );
            sender
                .send(trigger_action)
                .await
                .expect("Failed to send the request");
            rx.recv()
                .await
                .expect("Failed to send action")
                .expect("Failed to send trigger value");

            let (tx, mut rx) = channel(10);
            let get_action = DatabaseAction::Get(tx, "/root/new-test".to_string());
            sender
                .send(get_action)
                .await
                .expect("Failed to send the request");

            match rx.recv().await.expect("Failed to receive message") {
                Ok(_) => panic!("This key should not exist"),
                Err(e) => match e {
                    ErrorKind::InvalidKey(msg) => assert_eq!("Specified key does not exist", msg),
                    e => panic!("This is not a correct panic: {}", e),
                },
            }
        })
    }

    #[test]
    fn test_errors() -> Result<(), ErrorKind> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let mut db = Database::new("root".to_string())?;

            // Error #1
            match db
                .insert(
                    KeyType::Record("/other/status".to_string()),
                    ValueType::RecordPointer("ok".to_string()),
                )
                .await
            {
                Err(e) => match e {
                    ErrorKind::InvalidKey(msg) => {
                        assert_eq!("Key does not begin with the root table", msg)
                    }
                    _ => panic!("Should have returned InvalidKey instead {:?}", e),
                },
                Ok(_) => panic!("Returned with Ok but it should have with Err"),
            }

            // Error #2
            match db.get(KeyType::Record("/root/status".to_string())) {
                Err(e) => match e {
                    ErrorKind::InvalidKey(msg) => {
                        assert_eq!("Specified key does not exist", msg)
                    }
                    _ => panic!("Should have returned InvalidKey instead {:?}", e),
                },
                Ok(_) => panic!("Returned with Ok but it should have with Err"),
            }

            // Error #3
            match db.get(KeyType::Record("root/status".to_string())) {
                Err(e) => match e {
                    ErrorKind::InvalidKey(msg) => {
                        assert_eq!("Key must begin with '/' sign", msg)
                    }
                    _ => panic!("Should have returned InvalidKey instead {:?}", e),
                },
                Ok(_) => panic!("Returned with Ok but it should have with Err"),
            }

            // Error #4
            match db.delete_key(KeyType::Table("/root/asd".to_string())).await {
                Err(e) => match e {
                    ErrorKind::InvalidKey(msg) => {
                        assert_eq!("Parameter must be a Record type", msg)
                    }
                    _ => panic!("Should have returned InvalidKey instead {:?}", e),
                },
                Ok(_) => panic!("Returned with Ok but it should have with Err"),
            }

            // Error #5
            match db
                .delete_table(KeyType::Record("/root/asd".to_string()))
                .await
            {
                Err(e) => match e {
                    ErrorKind::InvalidKey(msg) => {
                        assert_eq!("Parameter must be a Table type", msg)
                    }
                    _ => panic!("Should have returned InvalidKey instead {:?}", e),
                },
                Ok(_) => panic!("Returned with Ok but it should have with Err"),
            }

            // Error #6
            match db
                .delete_key(KeyType::Record("/root/asd".to_string()))
                .await
            {
                Err(e) => match e {
                    ErrorKind::InvalidKey(msg) => {
                        assert_eq!("Specified key does not exist", msg)
                    }
                    _ => panic!("Should have returned InvalidKey instead {:?}", e),
                },
                Ok(_) => panic!("Returned with Ok but it should have with Err"),
            }

            // Error #7
            match db
                .delete_table(KeyType::Table("/root/asd".to_string()))
                .await
            {
                Err(e) => match e {
                    ErrorKind::InvalidKey(msg) => {
                        assert_eq!("Specified key does not exist", msg)
                    }
                    _ => panic!("Should have returned InvalidKey instead {:?}", e),
                },
                Ok(_) => panic!("Returned with Ok but it should have with Err"),
            }

            // Error #8
            match db.pop(KeyType::Record("/root/asd".to_string())).await {
                Err(e) => match e {
                    ErrorKind::InvalidKey(msg) => {
                        assert_eq!("Specified key does not exist", msg)
                    }
                    _ => panic!("Should have returned InvalidKey instead {:?}", e),
                },
                Ok(_) => panic!("Returned with Ok but it should have with Err"),
            }

            return Ok(());
        })
    }

    #[test]
    fn basic_functions() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let db = Database::new("root".to_string());
            assert_eq!(true, db.is_ok());

            let mut db = db.unwrap();

            // Insert some data
            let response = db
                .insert(
                    KeyType::Record("/root/status".to_string()),
                    ValueType::RecordPointer("okay".to_string()),
                )
                .await;
            assert_eq!(true, response.is_ok());

            let response = db
                .insert(
                    KeyType::Record("/root/status/sub1".to_string()),
                    ValueType::RecordPointer("okay".to_string()),
                )
                .await;
            assert_eq!(true, response.is_ok());

            let response = db
                .insert(
                    KeyType::Record("/root/status/sub2".to_string()),
                    ValueType::RecordPointer("okay".to_string()),
                )
                .await;
            assert_eq!(true, response.is_ok());

            let response = db
                .insert(
                    KeyType::Record("/root/node_name".to_string()),
                    ValueType::RecordPointer("teszt1".to_string()),
                )
                .await;
            assert_eq!(true, response.is_ok());

            let response = db
                .insert(
                    KeyType::Record("/root/network/gitea".to_string()),
                    ValueType::RecordPointer("okay".to_string()),
                )
                .await;
            assert_eq!(true, response.is_ok());

            // Check that value has been saved
            let value = db.get(KeyType::Record("/root/status".to_string()));
            assert_eq!(true, value.is_ok());

            let value = match value.unwrap() {
                ValueType::RecordPointer(value) => value,
                _ => panic!(),
            };
            assert_eq!("okay".to_string(), *value);

            // Get non exist key
            let response = db.get(KeyType::Record("/root/asd/eqq".to_string()));
            assert_eq!(true, response.is_err());

            // Check override value
            let response = db
                .insert(
                    KeyType::Record("/root/status".to_string()),
                    ValueType::RecordPointer("great".to_string()),
                )
                .await;
            assert_eq!(true, response.is_ok());

            match db.get(KeyType::Record("/root/status".to_string())) {
                Ok(value) => match value {
                    ValueType::RecordPointer(text) => assert_eq!("great".to_string(), *text),
                    _ => panic!("It should be record pointer"),
                },
                Err(e) => panic!("{}", e),
            }

            // Check some error
            let response = db
                .insert(
                    KeyType::Record("/status".to_string()),
                    ValueType::RecordPointer("okay".to_string()),
                )
                .await;
            assert_eq!(true, response.is_err());

            let response = db
                .insert(
                    KeyType::Record("root/batch/error/plan1".to_string()),
                    ValueType::RecordPointer("failed".to_string()),
                )
                .await;
            assert_eq!(true, response.is_err());

            // Check listing
            match db.list_keys(KeyType::Record("/root".to_string()), ListType::All) {
                Ok(table) => {
                    assert_eq!(true, table.len() >= 1);
                }
                Err(e) => panic!("{}", e),
            }

            match db.list_keys(KeyType::Record("/root/network".to_string()), ListType::All) {
                Ok(table) => {
                    assert_eq!(true, table.len() >= 1);
                }
                Err(e) => panic!("{}", e),
            }

            match db.list_keys(KeyType::Record("/root".to_string()), ListType::OneLevel) {
                Ok(table) => {
                    assert_eq!(true, table.len() >= 1);
                }
                Err(e) => panic!("{}", e),
            }

            // Try to list non-exist route
            let a = db.list_keys(KeyType::Record("/root/asd/eqq".to_string()), ListType::All);
            assert_eq!(true, a.is_err());

            // Delete key
            let response = db
                .delete_key(KeyType::Record("/root/status".to_string()))
                .await;
            assert_eq!(true, response.is_ok());

            let response = db.get(KeyType::Record("/root/status".to_string()));
            assert_eq!(true, response.is_err());

            let response = db
                .delete_key(KeyType::Record("/root/status".to_string()))
                .await;
            assert_eq!(true, response.is_err());

            // Drop table
            let response = db
                .delete_table(KeyType::Table("/root/status".to_string()))
                .await;
            assert_eq!(true, response.is_ok());

            let response = db.get(KeyType::Record("/root/status/sub1".to_string()));
            assert_eq!(true, response.is_err());

            // Add same name record and table pointer than queue to test that it is not a problem
            let response = db
                .insert(
                    KeyType::Record("/root/tickets".to_string()),
                    ValueType::RecordPointer("okay".to_string()),
                )
                .await;
            assert_eq!(true, response.is_ok());

            let response = db
                .insert(
                    KeyType::Record("/root/tickets/forward_to".to_string()),
                    ValueType::RecordPointer("127.0.0.1".to_string()),
                )
                .await;
            assert_eq!(true, response.is_ok());

            // Test queue
            let response = db
                .push(
                    KeyType::Record("/root/tickets/open".to_string()),
                    "SINC100".to_string(),
                )
                .await;
            assert_eq!(true, response.is_ok());

            let response = db
                .push(
                    KeyType::Record("/root/tickets/open".to_string()),
                    "SINC101".to_string(),
                )
                .await;
            assert_eq!(true, response.is_ok());

            let response = db
                .pop(KeyType::Record("/root/tickets/open".to_string()))
                .await
                .expect("Pop should work");
            assert_eq!("SINC100".to_string(), response);

            let response = db
                .pop(KeyType::Record("/root/tickets/open".to_string()))
                .await
                .expect("Pop should work");
            assert_eq!("SINC101".to_string(), response);

            let response = db
                .pop(KeyType::Record("/root/tickets/open".to_string()))
                .await;
            assert_eq!(true, response.is_err());

            // Test earlier gets again
            let value = db
                .get(KeyType::Record("/root/tickets".to_string()))
                .expect("Failed to fetch key after queue actions");
            assert_eq!(ValueType::RecordPointer("okay".to_string()), value);

            let value = db
                .get(KeyType::Record("/root/tickets/forward_to".to_string()))
                .expect("Failed to fetch key after queue actions");
            assert_eq!(ValueType::RecordPointer("127.0.0.1".to_string()), value);
        })
    }

    #[test]
    fn test_hook_manager() {
        let mut manager = HookManager::new();

        let result = manager.add(
            "/root/status".to_string(),
            "http://127.0.0.1:3031".to_string(),
        );
        assert_eq!(true, result.is_ok());

        let result = manager.add(
            "/root/status".to_string(),
            "http://127.0.0.1:3032".to_string(),
        );
        assert_eq!(true, result.is_ok());

        let result = manager.add(
            "/root/arpa".to_string(),
            "http://127.0.0.1:3031".to_string(),
        );
        assert_eq!(true, result.is_ok());

        let result = manager.list(&"/root".to_string());
        assert_eq!(2, result.len());

        let result = manager.list(&"/root/stat".to_string());
        assert_eq!(1, result.len());

        let result = manager.list(&"/root/no_exist".to_string());
        assert_eq!(0, result.len());

        // Start a dummy TCP listenere for testing
        std::thread::spawn(|| {
            let listener = std::net::TcpListener::bind("127.0.0.1:3031")
                .expect("Failed to listen on 127.0.0.1:3031");
            println!("Start to listen");
            while let Ok(stream) = listener.accept() {
                let mut stream = stream.0;
                stream.set_read_timeout(None).unwrap();
                let buf_reader = std::io::BufReader::new(&stream);
                let mut http_request = String::new();
                for byte in buf_reader.bytes() {
                    match byte {
                        Ok(byte) => {
                            let char = byte as char;
                            http_request.push(char);
                            stream
                                .set_read_timeout(Some(std::time::Duration::new(0, 250)))
                                .unwrap();
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                        Err(e) => {
                            println!("Unexpected error: {:?}", e);
                            let _ = stream.write_all(
                                b">Error\nInternal server error during stream reading\n",
                            );
                            panic!("TCP error");
                        }
                    }
                }

                println!("Request: {:#?}", http_request);
                stream
                    .write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes())
                    .unwrap();
            }
            panic!("TCP listener has stopped");
        });

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let counter = manager
                .execute_hooks(&"/root/status/dns1".to_string(), &"okay".to_string())
                .await;
            assert_eq!(Some(2), counter);

            let counter = manager
                .execute_hooks(&"/root/no_exist".to_string(), &"okay".to_string())
                .await;
            assert_eq!(None, counter);

            let counter = manager
                .execute_hooks(
                    &"/root/arpa/server1".to_string(),
                    &"This is the value".to_string(),
                )
                .await;
            assert_eq!(Some(1), counter);

            // Wait some time until request are received
            tokio::time::sleep(tokio::time::Duration::new(1, 0)).await;
        });
    }
}
