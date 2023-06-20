//! Enum for datastore

use crate::hook::types::{Link, Prefix};

use super::types::{
    ResultWithHook, ResultWithHooks, ResultWithList, ResultWithResult, ResultWithoutResult, Table,
};
use std::sync::mpsc::Sender;

pub mod error;
pub mod pair;

///
/// Specifiy the level for listing key function
///
#[derive(PartialEq, Clone)]
pub enum ListType {
    /// List only the current level
    OneLevel,

    /// List everything under it on recursive way
    All,
}

impl std::fmt::Display for ListType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::OneLevel => "OneLevel",
            Self::All => "All",
        };
        write!(f, "{}", text)
    }
}

///
/// Actions for built-in server
///
pub enum DatabaseAction {
    /// Set or update a key-value pair
    Set(Sender<ResultWithoutResult>, String, String),

    /// Get a value for a key
    Get(Sender<ResultWithResult>, String),

    /// Delete a pair
    DeleteKey(Sender<ResultWithoutResult>, String),

    /// Delete a whole table
    DeleteTable(Sender<ResultWithoutResult>, String),

    /// List keys from a route
    ListKeys(Sender<ResultWithList>, String, ListType),

    /// Send trigger to HookManager
    Trigger(Sender<ResultWithoutResult>, String, String),

    /// Set new hook
    HookSet(Sender<ResultWithoutResult>, Prefix, Link),

    /// Check that hook exist
    HookGet(Sender<ResultWithHook>, Prefix),

    /// Remove existing hook
    HookRemove(Sender<ResultWithoutResult>, Prefix, Link),

    /// List hooks
    HookList(Sender<ResultWithHooks>, Prefix),

    /// Command to suspend the logging
    SuspendLog(Sender<ResultWithoutResult>),

    /// Command to resume the logging
    ResumeLog(Sender<ResultWithoutResult>),

    /// Push to a queue
    Push(Sender<ResultWithoutResult>, String, String),

    /// Pop from queue
    Pop(Sender<ResultWithResult>, String),
}

impl std::fmt::Display for DatabaseAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Set(_, key, _) => format!("Set[{}]", key),
            Self::Get(_, key) => format!("Get[{}]", key),
            Self::DeleteKey(_, key) => format!("RemKey[{}]", key),
            Self::DeleteTable(_, key) => format!("RemPath[{}]", key),
            Self::ListKeys(_, key, r#type) => format!("ListKeys[{}, {}]", key, r#type),
            Self::Trigger(_, key, value) => format!("Trigger[{}, {}]", key, value),
            Self::HookSet(_, prefix, link) => format!("HookSet[{}, {}]", prefix, link),
            Self::HookGet(_, prefix) => format!("HookGet[{}]", prefix),
            Self::HookRemove(_, prefix, link) => format!("HookRemove[{}, {}]", prefix, link),
            Self::HookList(_, prefix) => format!("HookList[{}]", prefix),
            Self::SuspendLog(_) => "SuspendLog".to_string(),
            Self::ResumeLog(_) => "ResumeLog".to_string(),
            Self::Push(_, key, _) => format!("Push[{}]", key),
            Self::Pop(_, key) => format!("Pop[{}]", key),
        };
        write!(f, "{}", text)
    }
}
