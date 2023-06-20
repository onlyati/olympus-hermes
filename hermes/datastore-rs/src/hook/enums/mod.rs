use std::sync::mpsc::Sender;
use std::collections::BTreeMap;

use super::types::{Hooks, Key, Link, Prefix, Value};

#[derive(Debug)]
/// Input actions for HookManager
pub enum HookManagerAction {
    /// SET new hook
    Set(Sender<HookManagerResponse>, Prefix, Link),

    /// Remove existing hook
    Remove(Sender<HookManagerResponse>, Prefix, Link),

    /// Get that hook exist
    Get(Sender<HookManagerResponse>, Prefix),

    /// List hooks
    List(Sender<HookManagerResponse>, Prefix),

    /// Send data to defined hooks
    Send(Key, Value),
}

#[derive(Debug, Eq, PartialEq)]
/// Possible answers for HookManager
pub enum HookManagerResponse {
    /// Empty good response
    Ok,

    /// Somthing wrong happened
    Error(String),

    /// Reponse for GET
    Hook(Prefix, Hooks),

    /// Response for LIST
    HookList(BTreeMap<Prefix, Hooks>),
}
