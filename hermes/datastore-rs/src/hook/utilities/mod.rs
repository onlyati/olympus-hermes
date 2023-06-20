use std::sync::mpsc::{channel, Sender};
use std::thread::JoinHandle;

use super::enums::{HookManagerAction, HookManagerResponse};
use super::HookManager;

/// Start a HookManager on a single tokio thread
///
/// # Examples
/// ```
/// use onlyati_datastore::hook::utilities;
/// use onlyati_datastore::hook::enums::{HookManagerAction, HookManagerResponse};
/// use std::sync::mpsc::channel;
///
/// let (sender, _) = utilities::start_hook_manager();
///
/// let (tx, rx) = channel();
/// let action = HookManagerAction::Set(tx, "/root/stats".to_string(), "http://127.0.0.1:3031".to_string());
///
/// sender.send(action).expect("Failed to send request");
///
/// let response = rx.recv().expect("Failed to receive");
/// assert_eq!(HookManagerResponse::Ok, response);
///
/// ```
pub fn start_hook_manager() -> (Sender<HookManagerAction>, JoinHandle<()>) {
    let (tx, rx) = channel::<HookManagerAction>();
    let mut manager = HookManager::new();

    let thread = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to allocate runtime for HookManager");

        loop {
            match rx.recv() {
                Ok(request) => match request {
                    HookManagerAction::Set(sender, prefix, target) => {
                        match manager.add(prefix, target) {
                            Ok(_) => send_response!(sender, HookManagerResponse::Ok),
                            Err(e) => send_response!(sender, e),
                        }
                    }
                    HookManagerAction::Remove(sender, prefix, target) => {
                        match manager.remove(prefix, target) {
                            Ok(_) => send_response!(sender, HookManagerResponse::Ok),
                            Err(e) => send_response!(sender, e),
                        }
                    }
                    HookManagerAction::Get(sender, prefix) => match manager.get(&prefix) {
                        Some(hooks) => {
                            send_response!(sender, HookManagerResponse::Hook(prefix, hooks))
                        }
                        None => send_response!(
                            sender,
                            HookManagerResponse::Error("Not found".to_string())
                        ),
                    },
                    HookManagerAction::List(sender, prefix) => {
                        send_response!(
                            sender,
                            HookManagerResponse::HookList(manager.list(&prefix))
                        );
                    }
                    HookManagerAction::Send(test_key, value) => {
                        let manager = manager.clone();
                        rt.spawn(async move {
                            manager.execute_hooks(&test_key, &value).await;
                        });
                    }
                },
                Err(e) => panic!("Hook manager failed: {}", e),
            }
        }
    });

    (tx, thread)
}

macro_rules! send_response {
    ($sender:expr, $value:expr) => {
        $sender
            .send($value)
            .unwrap_or_else(|e| tracing::error!("Error during send: {}", e))
    };
}
pub(self) use send_response;
