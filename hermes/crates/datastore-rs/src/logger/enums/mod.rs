use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc::Sender;

use crate::datastore::enums::pair::KeyType;

/// Item for every action in datastore
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LogItem {
    SetKey(Duration, String, String),
    GetKey(Duration, String),
    RemKey(Duration, String),
    RemPath(Duration, String),
    ListKeys(Duration, String),
    Trigger(Duration, String, String),
    SetHook(Duration, String, String),
    GetHook(Duration, String),
    RemHook(Duration, String, String),
    ListHooks(Duration, String),
    HookExecute(Duration, String, Vec<String>),
    Push(Duration, String, String),
    Pop(Duration, String),
}

impl LogItem {
    pub fn needs_to_log(&self) -> bool {
        match &self {
            Self::SetKey(_, _, _) => true,
            Self::RemKey(_, _) => true,
            Self::RemPath(_, _) => true,
            Self::SetHook(_, _, _) => true,
            Self::RemHook(_, _, _) => true,
            Self::Push(_, _, _) => true,
            Self::Pop(_, _) => true,
            _ => false,
        }
    }

    pub fn is_rem_hook(&self) -> bool {
        matches!(&self, Self::RemHook(_, _, _))
    }

    pub fn get_key<'a>(&'a self) -> Option<KeyType> {
        match &self {
            Self::SetKey(_, key, _) => Some(KeyType::Record(key.to_string())),
            Self::RemKey(_, key) => Some(KeyType::Record(key.to_string())),
            Self::RemPath(_, key) => Some(KeyType::Record(key.to_string())),
            Self::SetHook(_, prefix, _) => Some(KeyType::Record(prefix.to_string())),
            Self::RemHook(_, prefix, _) => Some(KeyType::Record(prefix.to_string())),
            Self::Push(_, key, _) => Some(KeyType::Queue(key.to_string())),
            Self::Pop(_, key) => Some(KeyType::Queue(key.to_string())),
            _ => None,
        }
    }

    pub fn get_value<'a>(&'a self) -> &str {
        match &self {
            Self::SetKey(_, _, value) => value,
            Self::SetHook(_, _, value) => value,
            Self::RemHook(_, _, link) => link,
            Self::Push(_, _, value) => value,
            _ => "",
        }
    }

    pub fn get_duration(&self) -> u128 {
        match &self {
            Self::SetKey(dur, _, _) => dur.as_nanos(),
            Self::RemKey(dur, _) => dur.as_nanos(),
            Self::RemPath(dur, _) => dur.as_nanos(),
            Self::SetHook(dur, _, _) => dur.as_nanos(),
            Self::RemHook(dur, _, _) => dur.as_nanos(),
            Self::Push(dur, _, _) => dur.as_nanos(),
            Self::Pop(dur, _) => dur.as_nanos(),
            _ => 0,
        }
    }
}

impl std::fmt::Display for LogItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::SetKey(duration, key, value) => {
                format!("{} SetKey [ '{}', '{}' ]", duration.as_nanos(), key, value)
            }
            Self::GetKey(duration, key) => format!("{} GetKey [ '{}' ]", duration.as_nanos(), key),
            Self::RemKey(duration, key) => format!("{} RemKey [ '{}' ]", duration.as_nanos(), key),
            Self::RemPath(duration, key) => {
                format!("{} RemPath [ '{}' ]", duration.as_nanos(), key)
            }
            Self::ListKeys(duration, key) => {
                format!("{} ListKeys [ '{}' ]", duration.as_nanos(), key)
            }
            Self::Trigger(duration, key, value) => {
                format!("{} Trigger [ '{}', '{}' ]", duration.as_nanos(), key, value)
            }
            Self::SetHook(duration, prefix, link) => format!(
                "{} SetHook [ '{}', '{}' ]",
                duration.as_nanos(),
                prefix,
                link
            ),
            Self::GetHook(duration, prefix) => {
                format!("{} GetHook [ '{}' ]", duration.as_nanos(), prefix)
            }
            Self::RemHook(duration, prefix, link) => format!(
                "{} RemHook [ '{}', '{}' ]",
                duration.as_nanos(),
                prefix,
                link
            ),
            Self::ListHooks(duration, prefix) => {
                format!("{} ListHooks [ '{}' ]", duration.as_nanos(), prefix)
            }
            Self::HookExecute(duration, prefix, links) => format!(
                "{} HookExecute [ '{}', '{:?}' ]",
                duration.as_nanos(),
                prefix,
                links
            ),
            Self::Push(duration, key, value) => {
                format!("{} Push [ '{}', '{}' ]", duration.as_nanos(), key, value)
            }
            Self::Pop(duration, key) => format!("{} Pop [ '{}' ]", duration.as_nanos(), key),
        };
        write!(f, "{}", text)
    }
}

/// Represent state of logger
#[derive(PartialEq)]
pub enum LogState {
    /// File is closed, no write is possible
    Close,

    /// File is open, can be written directly
    Open,

    /// File is closed, but writes are buffered in memory
    Suspended,
}

/// Types that can be sent back by using the `start_logger` utility
#[derive(Debug, PartialEq)]
pub enum LoggerResponse {
    /// Request is successfully done
    Ok,

    FromAppendFile(Vec<LogItem>),

    /// Something is wrong, see in message
    Err(String),
}

/// Enums for the `start_logger` utility taht can be used with an std::sync::mpsc::Sender<LoggerAction> sender.
#[derive(Debug)]
pub enum LoggerAction {
    /// Close log file and buffer further message
    Suspend(Sender<LoggerResponse>),

    /// Open log file and write the buffered message
    Resume(Sender<LoggerResponse>),

    /// Write request
    Write(Sender<LoggerResponse>, Vec<LogItem>),
    WriteAsync(Vec<LogItem>),

    /// Read the append file
    ReadAppendFile(Sender<LoggerResponse>),
}

impl std::fmt::Display for LoggerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Resume(_) => "Resume".to_string(),
            Self::Suspend(_) => "Suspend".to_string(),
            Self::Write(_, item) => format!("Write [ '{:?}' ]", item),
            Self::WriteAsync(item) => format!("Write [ '{:?}' ]", item),
            Self::ReadAppendFile(_) => "ReadAppendFile".to_string(),
        };
        write!(f, "{}", text)
    }
}
