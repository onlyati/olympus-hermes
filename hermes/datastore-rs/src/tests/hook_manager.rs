#[cfg(test)]
mod tests {
    use std::{io::prelude::*, sync::mpsc::channel};

    use crate::{
        datastore::enums::DatabaseAction,
        hook::{utilities, HookManager},
    };

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
            "/root/status".to_string(),
            "http://127.0.0.1:3032".to_string(),
        );
        assert_eq!(true, result.is_err());

        let result = manager.add(
            "/root/status".to_string(),
            "http://127.0.0.1:3033".to_string(),
        );
        assert_eq!(true, result.is_ok());

        let result = manager.add(
            "/root/arpa".to_string(),
            "http://127.0.0.1:3031".to_string(),
        );
        assert_eq!(true, result.is_ok());

        let result = manager.remove(
            "/root/status".to_string(),
            "http://127.0.0.1:3033".to_string(),
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

    #[test]
    fn hook_manager_with_datastore() {
        let (sender, _) = utilities::start_hook_manager();
        let (sender, _) =
            crate::datastore::utilities::start_datastore("root".to_string(), Some(sender), None);

        // Add one hook
        let (tx, rx) = channel();
        let action = DatabaseAction::HookSet(
            tx,
            "/root/status".to_string(),
            "http://127.0.0.1:3031".to_string(),
        );
        sender.send(action).expect("Failed to send hook request");

        rx.recv()
            .expect("Failed to received response")
            .expect("Bad request");

        // Add another one hook
        let (tx, rx) = channel();
        let action = DatabaseAction::HookSet(
            tx,
            "/root/status".to_string(),
            "http://127.0.0.1:3032".to_string(),
        );
        sender.send(action).expect("Failed to send hook request");

        rx.recv()
            .expect("Failed to received response")
            .expect("Bad request");

        // Add a different one
        let (tx, rx) = channel();
        let action = DatabaseAction::HookSet(
            tx,
            "/root/arpa".to_string(),
            "http://127.0.0.1:3031".to_string(),
        );
        sender.send(action).expect("Failed to send hook request");

        rx.recv()
            .expect("Failed to received response")
            .expect("Bad request");

        // Test for get
        let (tx, rx) = channel();
        let action = DatabaseAction::HookGet(tx, "/root/status".to_string());
        sender.send(action).expect("Failed to send hook request");

        let list_etalon = vec![
            "http://127.0.0.1:3031".to_string(),
            "http://127.0.0.1:3032".to_string(),
        ];

        let result = rx
            .recv()
            .expect("Failed to received response")
            .expect("Bad request");
        assert_eq!("/root/status".to_string(), result.0);
        assert_eq!(2, result.1.len());
        assert_eq!(list_etalon, result.1);

        // Test for list
        let (tx, rx) = channel();
        let action = DatabaseAction::HookList(tx, "/root".to_string());
        sender.send(action).expect("Failed to send hook request");

        let result = rx
            .recv()
            .expect("Failed to received response")
            .expect("Bad request");
        assert_eq!(2, result.len());
        assert_eq!(true, result.contains_key(&"/root/status".to_string()));
        assert_eq!(true, result.contains_key(&"/root/arpa".to_string()));

        // Test remove
        let (tx, rx) = channel();
        let action = DatabaseAction::HookRemove(
            tx,
            "/root/arpa".to_string(),
            "http://127.0.0.1:3031".to_string(),
        );
        sender.send(action).expect("Failed to send hook request");

        let _result = rx
            .recv()
            .expect("Failed to received response")
            .expect("Bad request");

        // Test for list again
        let (tx, rx) = channel();
        let action = DatabaseAction::HookList(tx, "/root".to_string());
        sender.send(action).expect("Failed to send hook request");

        let result = rx
            .recv()
            .expect("Failed to received response")
            .expect("Bad request");
        println!("{:?}", result);
        assert_eq!(1, result.len());
        assert_eq!(true, result.contains_key(&"/root/status".to_string()));
        assert_eq!(false, result.contains_key(&"/root/arpa".to_string()));

        // Test remove again
        let (tx, rx) = channel();
        let action = DatabaseAction::HookRemove(
            tx,
            "/root/status".to_string(),
            "http://127.0.0.1:3031".to_string(),
        );
        sender.send(action).expect("Failed to send hook request");

        let _result = rx
            .recv()
            .expect("Failed to received response")
            .expect("Bad request");

        // Test get again
        let (tx, rx) = channel();
        let action = DatabaseAction::HookGet(tx, "/root/status".to_string());
        sender.send(action).expect("Failed to send hook request");

        let list_etalon = vec!["http://127.0.0.1:3032".to_string()];

        let result = rx
            .recv()
            .expect("Failed to received response")
            .expect("Bad request");
        assert_eq!("/root/status".to_string(), result.0);
        assert_eq!(1, result.1.len());
        assert_eq!(list_etalon, result.1);
    }
}
