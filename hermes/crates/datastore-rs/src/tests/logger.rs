#[cfg(test)]
mod test {
    use tokio::sync::mpsc::channel;

    use crate::{
        datastore::{enums::DatabaseAction, utilities::start_datastore},
        logger::{
            enums::{LogItem, LoggerAction, LoggerResponse},
            utilities::start_logger,
            LoggerManager,
        },
    };

    #[test]
    fn test_log1() {
        let path = "/tmp/datastore-log1".to_string();
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("failed to delete directory");

        let placeholder_date = std::time::Duration::from_secs(5);

        let mut manager = LoggerManager::new(Some(path));

        let result = manager.start(false);
        assert_eq!(true, result.is_ok());

        let result = manager.write(LogItem::SetKey(
            placeholder_date,
            "/root/status/server1".to_string(),
            "alive".to_string(),
        ));
        assert_eq!(true, result.is_ok());

        let result = manager.stop(false);
        assert_eq!(true, result.is_ok());

        let result = manager.write(LogItem::SetKey(
            placeholder_date,
            "/root/status/server9".to_string(),
            "alive".to_string(),
        ));
        assert_eq!(true, result.is_err());
    }

    #[test]
    fn test_log2() {
        let path = "/tmp/datastore-log2".to_string();
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("failed to delete directory");

        let placeholder_date = std::time::Duration::from_secs(5);

        // Start logger and write a line
        let mut manager = LoggerManager::new(Some(path.clone()));

        let result = manager.start(false);
        assert_eq!(true, result.is_ok());

        let result = manager.write(LogItem::SetKey(
            placeholder_date,
            "/root/tickets/345".to_string(),
            "open".to_string(),
        ));
        assert_eq!(true, result.is_ok());

        let result = manager.write(LogItem::SetKey(
            placeholder_date,
            "/root/tickets/346".to_string(),
            "open".to_string(),
        ));
        assert_eq!(true, result.is_ok());

        // Suspend the logger: file is closed and every message will be buffered
        let result = manager.suspend();
        assert_eq!(true, result.is_ok());

        let human_log = format!("{}/human.log", path);

        let content =
            std::fs::read_to_string(&human_log).expect("Failed to open file for line counting");
        let count1: Vec<&str> = content.lines().collect();
        let count1 = count1.len();

        // Still should be ok, but instead of file, it is buffered in memory
        let result = manager.write(LogItem::SetKey(
            placeholder_date,
            "/root/tickets/345".to_string(),
            "close".to_string(),
        ));
        assert_eq!(true, result.is_ok());

        // Let's check the number of lines in file, it should be the same
        let content =
            std::fs::read_to_string(&human_log).expect("Failed to open file for line counting");
        let count2: Vec<&str> = content.lines().collect();
        let count2 = count2.len();

        assert_eq!(count1, count2);

        // Now make a resume then close the file (so it can be read)
        let result = manager.resume();
        assert_eq!(true, result.is_ok());

        // Check line numbers again, should be more with one
        let content =
            std::fs::read_to_string(human_log).expect("Failed to open file for line counting");
        let count3: Vec<&str> = content.lines().collect();
        let count3 = count3.len();

        assert_eq!(count2 + 1, count3);
    }

    #[test]
    fn test_log3() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let path = "/tmp/datastore-log3".to_string();
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("failed to delete directory");

            let human_log = format!("{}/human.log", path);

            let placeholder_date = std::time::Duration::from_secs(5);

            let (sender, _) = start_logger(Some(path)).await;

            let action = LoggerAction::WriteAsync(vec![LogItem::SetKey(
                placeholder_date,
                "/root/test1".to_string(),
                "something".to_string(),
            )]);
            sender
                .send(action)
                .await
                .expect("Failed to send the request");

            let (tx, mut rx) = channel(10);

            let action = LoggerAction::Write(
                tx,
                vec![
                    LogItem::GetKey(placeholder_date, "/root/test/1".to_string()),
                    LogItem::GetKey(placeholder_date, "/root/test/2".to_string()),
                    LogItem::GetKey(placeholder_date, "/root/test/3".to_string()),
                ],
            );
            sender
                .send(action)
                .await
                .expect("Failed to send the request");

            let response = rx.recv().await.expect("Failed to receive reply");
            assert_eq!(LoggerResponse::Ok, response);

            let content =
                std::fs::read_to_string(human_log).expect("Failed to open file for line counting");
            let count: Vec<&str> = content.lines().collect();
            let count = count.len();

            assert_eq!(4, count);
        });
    }

    #[test]
    fn test_log4() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let path = "/tmp/datastore-log4".to_string();
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("failed to delete directory");

            let human_log = format!("{}/human.log", path);

            let (logger_sender, _) = start_logger(Some(path)).await;
            let (sender, _) = start_datastore("root".to_string(), None, Some(logger_sender)).await;

            let (tx, mut rx) = channel(10);
            let action =
                DatabaseAction::Set(tx, "/root/test1".to_string(), "available".to_string());

            sender
                .send(action)
                .await
                .expect("Failed to send get requst");
            rx.recv()
                .await
                .expect("Failed to receive")
                .expect("Failed to set the value");

            std::thread::sleep(std::time::Duration::new(6, 0)); // Wait some time that the async write will be finished

            // Check that log write really happened
            let content =
                std::fs::read_to_string(&human_log).expect("Failed to open file for line counting");
            let count: Vec<&str> = content.lines().collect();
            let count = count.len();

            assert_eq!(1, count);

            // Suspend the logging and check that no write happen
            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::SuspendLog(tx);

            sender
                .send(action)
                .await
                .expect("Failed to send suspend request");

            rx.recv()
                .await
                .expect("Failed to receive message")
                .expect("Failed to suspend logging");

            let (tx, mut rx) = channel(10);
            let action =
                DatabaseAction::Set(tx, "/root/test1".to_string(), "available".to_string());

            sender
                .send(action)
                .await
                .expect("Failed to send get requst");
            rx.recv()
                .await
                .expect("Failed to receive")
                .expect("Failed to set the value");

            std::thread::sleep(std::time::Duration::new(6, 0)); // Wait some time that the async write will be finished

            let content =
                std::fs::read_to_string(&human_log).expect("Failed to open file for line counting");
            let count2: Vec<&str> = content.lines().collect();
            let count2 = count2.len();

            assert_eq!(count, count2);

            // Now do a resume and check log lines in file
            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::ResumeLog(tx);

            sender
                .send(action)
                .await
                .expect("Failed to send suspend request");

            rx.recv()
                .await
                .expect("Failed to receive message")
                .expect("Failed to suspend logging");

            let content = std::fs::read_to_string(human_log.clone())
                .expect("Failed to open file for line counting");
            let count2: Vec<&str> = content.lines().collect();
            let count2 = count2.len();

            assert_eq!(count + 1, count2);
        });
    }

    #[test]
    fn test_log5() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let path = "/tmp/datastore-log5".to_string();
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("failed to delete directory");

            let placeholder_date = std::time::Duration::from_secs(5);

            let etalon = vec![
                LogItem::SetKey(placeholder_date, "/root/test1".to_string(), "placeholder value".to_string()),
                LogItem::SetKey(placeholder_date, "/root/test2".to_string(), "placeholder value".to_string())
            ];

            let (logger_sender, _) = start_logger(Some(path)).await;

            let (tx, mut rx) = channel(10);
            let action = LoggerAction::Write(tx, etalon.clone());

            logger_sender.send(action).await.unwrap();
            rx.recv().await.unwrap();

            let (tx, mut rx) = channel(10);
            let action = LoggerAction::ReadAppendFile(tx);

            logger_sender.send(action).await.unwrap();
            let response = rx.recv().await.unwrap();
            assert_eq!(LoggerResponse::FromAppendFile(etalon), response);
        });
    }
}
