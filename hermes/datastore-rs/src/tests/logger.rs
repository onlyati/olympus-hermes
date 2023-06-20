#[cfg(test)]
mod test {
    use std::path::Path;
    use std::sync::mpsc::channel;

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
        let path = "/tmp/datastore-log.txt".to_string();
        {
            let path = Path::new("/tmp/datastore-log.txt");
            if path.exists() {
                std::fs::remove_file(path).expect("Failed to delete temp log");
            }
        }

        let mut manager = LoggerManager::new(path);

        let result = manager.start();
        assert_eq!(true, result.is_ok());

        let result = manager.write(LogItem::SetKey(
            "/root/status/server1".to_string(),
            "alive".to_string(),
        ));
        assert_eq!(true, result.is_ok());

        let result = manager.stop();
        assert_eq!(true, result.is_ok());

        let result = manager.write(LogItem::SetKey(
            "/root/status/server9".to_string(),
            "alive".to_string(),
        ));
        assert_eq!(true, result.is_err());
    }

    #[test]
    fn test_log2() {
        let path = "/tmp/datastore-log2.txt".to_string();
        {
            let path = Path::new(&path);
            if path.exists() {
                std::fs::remove_file(path).expect("Failed to delete temp log");
            }
        }

        // Start logger and write a line
        let mut manager = LoggerManager::new(path.clone());

        let result = manager.start();
        assert_eq!(true, result.is_ok());

        let result = manager.write(LogItem::SetKey(
            "/root/tickets/345".to_string(),
            "open".to_string(),
        ));
        assert_eq!(true, result.is_ok());

        let result = manager.write(LogItem::SetKey(
            "/root/tickets/346".to_string(),
            "open".to_string(),
        ));
        assert_eq!(true, result.is_ok());

        // Suspend the logger: file is closed and every message will be buffered
        let result = manager.suspend();
        assert_eq!(true, result.is_ok());

        let content =
            std::fs::read_to_string(path.clone()).expect("Failed to open file for line counting");
        let count1: Vec<&str> = content.lines().collect();
        let count1 = count1.len();

        // Still should be ok, but instead of file, it is buffered in memory
        let result = manager.write(LogItem::SetKey(
            "/root/tickets/345".to_string(),
            "close".to_string(),
        ));
        assert_eq!(true, result.is_ok());

        // Let's check the number of lines in file, it should be the same
        let content =
            std::fs::read_to_string(path.clone()).expect("Failed to open file for line counting");
        let count2: Vec<&str> = content.lines().collect();
        let count2 = count2.len();

        assert_eq!(count1, count2);

        // Now make a resume then close the file (so it can be read)
        let result = manager.resume();
        assert_eq!(true, result.is_ok());

        // Check line numbers again, should be more with one
        let content =
            std::fs::read_to_string(path.clone()).expect("Failed to open file for line counting");
        let count3: Vec<&str> = content.lines().collect();
        let count3 = count3.len();

        assert_eq!(count2 + 1, count3);
    }

    #[test]
    fn test_log3() {
        let path = Path::new("/tmp/datastore-log3.txt");
        if path.exists() {
            std::fs::remove_file(path).expect("Failed to delete temp log");
        }

        let (sender, _) = start_logger(&"/tmp/datastore-log3.txt".to_string());

        let action = LoggerAction::WriteAsync(vec![LogItem::SetKey(
            "/root/test1".to_string(),
            "something".to_string(),
        )]);
        sender.send(action).expect("Failed to send the request");

        let (tx, rx) = channel();

        let action = LoggerAction::Write(
            tx,
            vec![
                LogItem::GetKey("/root/test/1".to_string()),
                LogItem::GetKey("/root/test/2".to_string()),
                LogItem::GetKey("/root/test/3".to_string()),
            ],
        );
        sender.send(action).expect("Failed to send the request");

        let response = rx.recv().expect("Failed to receive reply");
        assert_eq!(LoggerResponse::Ok, response);

        let content = std::fs::read_to_string(path).expect("Failed to open file for line counting");
        let count: Vec<&str> = content.lines().collect();
        let count = count.len();

        assert_eq!(4, count);
    }

    #[test]
    fn test_log4() {
        let path = "/tmp/datastre-log4.txt".to_string();
        {
            let path = Path::new(&path);
            if path.exists() {
                std::fs::remove_file(path).expect("Failed to delete temp log");
            }
        }

        let (logger_sender, _) = start_logger(&path);
        let (sender, _) = start_datastore("root".to_string(), None, Some(logger_sender));

        let (tx, rx) = channel();
        let action = DatabaseAction::Set(tx, "/root/test1".to_string(), "available".to_string());

        sender.send(action).expect("Failed to send get requst");
        rx.recv()
            .expect("Failed to receive")
            .expect("Failed to set the value");

        std::thread::sleep(std::time::Duration::new(1, 0)); // Wait some time that the async write will be finished

        // Check that log write really happened
        let content =
            std::fs::read_to_string(path.clone()).expect("Failed to open file for line counting");
        let count: Vec<&str> = content.lines().collect();
        let count = count.len();

        assert_eq!(1, count);

        // Suspend the logging and check that no write happen
        let (tx, rx) = channel();
        let action = DatabaseAction::SuspendLog(tx);

        sender.send(action).expect("Failed to send suspend request");

        rx.recv()
            .expect("Failed to receive message")
            .expect("Failed to suspend logging");

        let (tx, rx) = channel();
        let action = DatabaseAction::Set(tx, "/root/test1".to_string(), "available".to_string());

        sender.send(action).expect("Failed to send get requst");
        rx.recv()
            .expect("Failed to receive")
            .expect("Failed to set the value");

        std::thread::sleep(std::time::Duration::new(1, 0)); // Wait some time that the async write will be finished

        let content =
            std::fs::read_to_string(path.clone()).expect("Failed to open file for line counting");
        let count2: Vec<&str> = content.lines().collect();
        let count2 = count2.len();

        assert_eq!(count, count2);

        // Now do a resume and check log lines in file
        let (tx, rx) = channel();
        let action = DatabaseAction::ResumeLog(tx);

        sender.send(action).expect("Failed to send suspend request");

        rx.recv()
            .expect("Failed to receive message")
            .expect("Failed to suspend logging");

        let content =
            std::fs::read_to_string(path.clone()).expect("Failed to open file for line counting");
        let count2: Vec<&str> = content.lines().collect();
        let count2 = count2.len();

        assert_eq!(count + 1, count2);
    }
}
