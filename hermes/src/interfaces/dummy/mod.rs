// External dependencies
use std::thread::JoinHandle;

// Internal depenencies
use super::ApplicationInterface;

/// This sturct is for those thread that are started by something else but monitored by InterfaceHandler
pub struct Dummy {
    thread: Option<JoinHandle<()>>,
}

impl Dummy {
    /// Create a new interface
    pub fn new(thread: Option<JoinHandle<()>>) -> Self {
        return Self { thread };
    }
}

impl ApplicationInterface for Dummy {
    fn run(&mut self) {}

    fn is_it_run(&self) -> Option<bool> {
        match &self.thread {
            Some(thread) => Some(!thread.is_finished()),
            None => None,
        }
    }
}
