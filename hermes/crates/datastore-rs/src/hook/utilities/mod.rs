use tokio::sync::mpsc::{channel, Sender};
use tokio::task::JoinHandle;

use super::enums::{HookManagerAction, HookManagerResponse};
use super::HookManager;

/// Start a HookManager on a single tokio thread
///
/// # Examples
/// ```
/// use onlyati_datastore::hook::utilities;
/// use onlyati_datastore::hook::enums::{HookManagerAction, HookManagerResponse};
/// use tokio::sync::mpsc::channel;
///
/// # tokio_test::block_on(async {
/// let (sender, _) = utilities::start_hook_manager().await;
///
/// let (tx, mut rx) = channel(10);
/// let action = HookManagerAction::Set(tx, "/root/stats".to_string(), "http://127.0.0.1:3031".to_string());
///
/// sender.send(action).await.expect("Failed to send request");
///
/// let response = rx.recv().await.expect("Failed to receive");
/// assert_eq!(HookManagerResponse::Ok, response);
/// # })
/// ```
pub async fn start_hook_manager() -> (Sender<HookManagerAction>, JoinHandle<()>) {
    let (tx, mut rx) = channel::<HookManagerAction>(100);
    let mut manager = HookManager::new();

    let thread = tokio::spawn(async move {
        while let Some(received) = rx.recv().await {
            match received {
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
                    None => {
                        send_response!(sender, HookManagerResponse::Error("Not found".to_string()))
                    }
                },
                HookManagerAction::List(sender, prefix) => {
                    send_response!(sender, HookManagerResponse::HookList(manager.list(&prefix)));
                }
                HookManagerAction::Send(test_key, value) => {
                    manager.execute_hooks(&test_key, &value).await;
                }
                HookManagerAction::Enable(sender, enable) => {
                    manager.enable(enable);
                    send_response!(sender, HookManagerResponse::Ok);
                }
            }
        }
    });

    (tx, thread)
}

macro_rules! send_response {
    ($sender:expr, $value:expr) => {
        $sender
            .send($value)
            .await
            .unwrap_or_else(|e| tracing::error!("Error during send: {}", e))
    };
}
pub(self) use send_response;
