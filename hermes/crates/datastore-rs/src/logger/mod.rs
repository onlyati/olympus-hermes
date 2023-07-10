use chrono::{DateTime, Utc};
use std::{
    collections::VecDeque,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use self::enums::{LogItem, LogState};

pub mod enums;
pub mod utilities;

/// Logger manager main structure
///
/// There are 3 state fo logger:
/// - Open: File is open, so direct write is possible
/// - Close: File is closed, so direct write is not possible and every write request will be dismissed
/// - Suspended: File is closed, and every further write request will be buffered. Once it is resumed, those records will be written
///
/// # Examples
/// ```
/// use std::path::Path;
/// use onlyati_datastore::logger::{LoggerManager, enums::LogItem};
///
/// // Create a new logger instance that will run into a temp file
/// // Normally this point to a file that is not temporary
/// let path = "/tmp/datastore-log-temp".to_string();
/// let _ = std::fs::remove_dir_all(&path);
/// std::fs::create_dir_all(&path).expect("failed to delete directory");
/// let mut logger = LoggerManager::new(path);
/// ```
///
/// For more details check `src/tests/logger.rs` file.
pub struct LoggerManager {
    /// Path to a directory where log files can be put
    /// If empty then logging is disabled
    path: String,

    /// Represent status of logging, used for suspend-resume actions
    pub(crate) state: LogState,

    /// Internal buffer for human log file
    human_log_file: Option<BufWriter<File>>,

    /// Buffer used when logging is suspended
    buffer: Vec<(DateTime<Utc>, LogItem)>,

    /// Internal buffer for logging because log is not written for every
    /// single item, but when a huge amount of log item has been receieved
    /// or some time has pass without update
    pub(crate) write_buffer: VecDeque<LogItem>,
}

impl LoggerManager {
    /// Allocate new logger
    ///
    /// # Arguments
    /// 1. `path`: File location where the logger file is written
    ///
    /// # Panic
    /// 
    /// If the log directory does not exist and not able to create
    /// 
    /// # Return
    ///
    /// Witha LoggerManager struct.
    pub fn new(path: String) -> Self {
        tracing::trace!("allocate new log manager with '{:?}' path", path);

        let log_dir = Path::new(&path);
        if !log_dir.exists() {
            tracing::info!("logger directory does not exist, try to create");
            match std::fs::create_dir_all(&log_dir) {
                Ok(_) => tracing::info!("logger directory '{}' successfully created", path),
                Err(e) => panic!("{}", e),
            }
        }

        Self {
            path: path.clone(),
            state: LogState::Close,
            human_log_file: None,
            buffer: Vec::new(),
            write_buffer: VecDeque::new(),
        }
    }

    /// Write binary append file and the human log.
    ///
    /// # Return
    ///
    /// With Ok if everything went fine else with an error message.
    /// If logging is disabled return with Ok.
    pub fn write_append_file(&mut self) -> Result<(), String> {
        if self.path.is_empty() {
            return Ok(());
        }

        let file_name = format!("{}/hermes.af", self.path);

        tracing::trace!("opening {} append file for write", file_name);

        let mut buffer = match File::options()
            .create(true)
            .write(true)
            .read(true)
            .append(true)
            .open(Path::new(&file_name))
        {
            Ok(file) => BufWriter::new(file),
            Err(e) => return Err(e.to_string()),
        };

        self.start(false).unwrap();

        while let Some(item) = &self.write_buffer.pop_front() {
            if item.needs_to_log() {
                tracing::trace!("item is logged: {:?}", item);
                let encoded_item: Vec<u8> = match bincode::serialize(&item) {
                    Ok(vec) => vec,
                    Err(e) => return Err(e.to_string()),
                };

                if let Err(e) = buffer.write_all(&encoded_item) {
                    return Err(e.to_string());
                }
            }

            self.write(item.clone()).unwrap();
        }

        buffer.flush().unwrap();

        tracing::trace!("close {} append file", file_name);

        self.stop(false).unwrap();
        Ok(())
    }

    /// Open a buffer for the specified file.
    /// After it, every write request will be directly written to file.
    ///
    /// # Return
    ///
    /// With Ok or with an error text.
    pub fn start(&mut self, force: bool) -> Result<(), String> {
        let file_name = format!("{}/human.log", self.path);

        if self.state == LogState::Suspended && !force {
            return Ok(());
        }

        tracing::trace!("opening file for write");
        match File::options()
            .create(true)
            .write(true)
            .read(true)
            .append(true)
            .open(Path::new(&file_name))
        {
            Ok(file) => {
                tracing::trace!("log file is open");
                self.human_log_file = Some(BufWriter::new(file));
                self.state = LogState::Open;
                Ok(())
            }
            Err(e) => {
                tracing::error!("failed to open log file: {}", e);
                Err(format!("Failed to open log file: {}", e))
            }
        }
    }

    /// Close the buffer for the specified file.
    /// After it, every write request is going to be failed and not buffered.
    ///
    /// # Return
    ///
    /// With Ok or with an error text.
    pub fn stop(&mut self, force: bool) -> Result<(), String> {
        if self.state == LogState::Suspended && !force {
            return Ok(());
        }

        match &mut self.human_log_file {
            Some(buffer) => {
                if let Err(e) = buffer.flush() {
                    return Err(e.to_string());
                }
                self.human_log_file = None;
                tracing::trace!("closed the log file");
                Ok(())
            }
            None => Err(String::from("Logger manager does not run")),
        }
    }

    /// Close the log file and do not write more into it.
    /// Instead buffer every message into memory.
    /// They will be written if the logging has resumed.
    ///
    /// # Return
    ///
    /// With Ok or with an error text.
    pub fn suspend(&mut self) -> Result<(), String> {
        tracing::trace!("suspend the logging");
        if self.human_log_file.is_some() {
            self.human_log_file = None;
        }
        self.state = LogState::Suspended;

        Ok(())
    }

    /// Resume the logging means that those message which were buffered during suspended status will be written first.
    /// Then status will be LogState::Open again.
    ///
    /// # Return
    ///
    /// With Ok or with an error text.
    pub fn resume(&mut self) -> Result<(), String> {
        tracing::trace!("resume the logging");
        if self.state != LogState::Suspended {
            return Err(String::from("Only possible resume from LogState::Suspend"));
        }

        self.start(true)?;

        tracing::trace!("writing {} lines after resume", self.buffer.len());

        for item in &self.buffer {
            if let Some(file) = &mut self.human_log_file {
                let line = format!("{} {}\n", item.0, item.1);
                if let Err(e) = file.write_all(line.as_bytes()) {
                    tracing::error!("failed to write log after a resume: {}", e);
                    return Err(format!("Failed to write log after a resume: {}", e));
                }
            }
        }

        self.buffer = Vec::new();

        self.stop(true)?;

        tracing::trace!("logging has resumed");
        Ok(())
    }

    /// Make a write reqest
    ///
    /// # Arguments
    /// 1. `item`: Item that will be written onto the log file
    ///
    /// # Return
    ///
    /// With Ok or with an error text.
    pub fn write(&mut self, item: LogItem) -> Result<(), String> {
        tracing::trace!("write log record");
        let now = Utc::now();

        match &mut self.state {
            // Logger is stopped
            LogState::Close => {
                tracing::trace!("write is abandoned due to it is not open");
                Err(String::from("Stream is closed, start required for logger"))
            }
            // Regular write to a file
            LogState::Open => match &mut self.human_log_file {
                Some(file) => {
                    let line = item.to_string() + "\n";
                    match file.write_all(line.as_bytes()) {
                        Ok(_) => {
                            tracing::trace!("write is done");
                            Ok(())
                        }
                        Err(e) => {
                            tracing::error!("error during log writing: {}", e);
                            Err(format!("error during log writing: {}", e))
                        }
                    }
                }
                None => {
                    tracing::error!("wanted to write log while logging was not started");
                    Err(String::from(
                        "wanted to write log while logging was not started",
                    ))
                }
            },
            // Buffer lines into memory
            LogState::Suspended => {
                self.buffer.push((now, item));
                tracing::trace!("write is done in suspended mode");
                Ok(())
            }
        }
    }
}
