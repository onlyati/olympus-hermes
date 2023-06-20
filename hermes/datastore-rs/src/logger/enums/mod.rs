use std::sync::mpsc::Sender;

/// Item for every action in datastore
#[derive(Clone, Debug)]
pub enum LogItem {
    SetKey(String, String),
    GetKey(String),
    RemKey(String),
    RemPath(String),
    ListKeys(String),
    Trigger(String, String),
    SetHook(String, String),
    GetHook(String),
    RemHook(String, String),
    ListHooks(String),
    HookExecute(String, Vec<String>),
    Push(String, String),
    Pop(String),
}

impl std::fmt::Display for LogItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::SetKey(key, value) => format!("SetKey [ '{}', '{}' ]", key, value),
            Self::GetKey(key) => format!("GetKey [ '{}' ]", key),
            Self::RemKey(key) => format!("RemKey [ '{}' ]", key),
            Self::RemPath(key) => format!("RemPath [ '{}' ]", key),
            Self::ListKeys(key) => format!("ListKeys [ '{}' ]", key),
            Self::Trigger(key, value) => format!("Trigger [ '{}', '{}' ]", key, value),
            Self::SetHook(prefix, link) => format!("SetHook [ '{}', '{}' ]", prefix, link),
            Self::GetHook(prefix) => format!("GetHook [ '{}' ]", prefix),
            Self::RemHook(prefix, link) => format!("RemHook [ '{}', '{}' ]", prefix, link),
            Self::ListHooks(prefix) => format!("ListHooks [ '{}' ]", prefix),
            Self::HookExecute(prefix, links) => format!("HookExecute [ '{}', '{:?}' ]", prefix, links),
            Self::Push(key, value) => format!("Push [ '{}', '{}' ]", key, value),
            Self::Pop(key) => format!("Pop [ '{}' ]", key),
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
#[derive(PartialEq, Debug)]
pub enum LoggerResponse {
    /// Request is successfully done
    Ok,

    /// Something is wrong, see in message
    Err(String),
}

/// Enums for the `start_logger` utility taht can be used with an std::sync::mpsc::Sender<LoggerAction> sender.
pub enum LoggerAction {
    /// Close log file and buffer further message
    Suspend(Sender<LoggerResponse>),

    /// Open log file and write the buffered message
    Resume(Sender<LoggerResponse>),

    /// Write request
    Write(Sender<LoggerResponse>, Vec<LogItem>),
    WriteAsync(Vec<LogItem>),
}

impl std::fmt::Display for LoggerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Resume(_) => "Resume".to_string(),
            Self::Suspend(_) => "Suspend".to_string(),
            Self::Write(_, item) => format!("Write [ '{:?}' ]", item),
            Self::WriteAsync(item) => format!("Write [ '{:?}' ]", item),
        };
        write!(f, "{}", text)
    }
}
