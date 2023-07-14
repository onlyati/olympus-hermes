//! Built-in utilities

use tokio::sync::mpsc::{channel, Sender};
use tokio::task::JoinHandle;

pub(crate) mod internal;

use crate::{
    hook::enums::{HookManagerAction, HookManagerResponse},
    logger::enums::{LogItem, LoggerAction, LoggerResponse},
};

use super::{
    enums::{error::ErrorKind, pair::KeyType, pair::ValueType, DatabaseAction, ListType},
    types::Table,
    Database,
};

/// Initialize database on another thread, create a channel and return with it
/// For all possible action check `onlyati::datastore::enums::mod::DatabaseAction` enum.
///
/// # Example for call
///
/// ```
/// use onlyati_datastore::datastore::{
///     enums::{error::ErrorKind, DatabaseAction, pair::ValueType},
///     utilities::start_datastore,
/// };
/// use tokio::sync::mpsc::channel;
///
/// # tokio_test::block_on(async {
/// let (sender, _) = start_datastore("root".to_string(), None, None).await;
///
/// // Add a new pair
/// let (tx, mut rx) = channel(10);
/// let set_action = DatabaseAction::Set(tx, "/root/network".to_string(), "ok".to_string());
///
/// sender.send(set_action).await.expect("Failed to send the request");
/// rx.recv().await.unwrap();
///
/// // Get the pair
/// let (tx, mut rx) = channel(10);
/// let get_action = DatabaseAction::Get(tx, "/root/network".to_string());
///
/// sender.send(get_action).await.expect("Failed to send the get request");
/// let data = rx.recv().await.expect("Failed to receive message").expect("Failed to get data");
/// assert_eq!(ValueType::RecordPointer("ok".to_string()), data);
/// # })
/// ```
pub async fn start_datastore(
    name: String,
    hook_sender: Option<Sender<HookManagerAction>>,
    logger_sender: Option<Sender<LoggerAction>>,
) -> (Sender<DatabaseAction>, JoinHandle<()>) {
    tracing::debug!("root element of database is '{}'", name);
    let (tx, mut rx) = tokio::sync::mpsc::channel::<DatabaseAction>(100);

    let thread = tokio::spawn(async move {
        let mut db = Database::new(name).expect("Failed to allocate database");

        //
        // Set hook manager
        //
        if let Some(sender) = hook_sender {
            tracing::debug!("subscribed to a hook manager");
            db.subscribe_to_hook_manager(sender);
        }

        //
        // Set logger
        //
        if let Some(sender) = logger_sender {
            // Turn off hook execution because it is just a recovery
            tracing::debug!("turn off hook execution during data restore");
            if let Some(hook_sender) = &db.hook_sender {
                let (tx, mut rx) = channel(10);
                let action = HookManagerAction::Enable(tx, false);

                hook_sender.send(action).await.unwrap();

                let a = rx.recv().await.unwrap();
                if a != HookManagerResponse::Ok {
                    tracing::error!("failed to turn off hook exectuin: {:?}", a);
                    return;
                }
            }

            // Start the recovery process
            tracing::debug!("start to restore data from append file");
            let (tx, mut rx) = channel(10);
            let action = LoggerAction::ReadAppendFile(tx);
            send_response_with_mutex_sender!(sender, action);

            match rx.recv().await {
                Some(list) => match list {
                    LoggerResponse::FromAppendFile(list) => {
                        tracing::info!(
                            "read {} item from append file, started to process them",
                            list.len()
                        );
                        for action in list {
                            match action {
                                LogItem::SetKey(_, key, value) => {
                                    db.insert(
                                        KeyType::Record(key),
                                        ValueType::RecordPointer(value),
                                    )
                                    .await
                                    .unwrap();
                                }
                                LogItem::RemKey(_, key) => {
                                    db.delete_key(KeyType::Record(key)).await.unwrap();
                                }
                                LogItem::RemPath(_, key) => {
                                    db.delete_table(KeyType::Record(key)).await.unwrap();
                                }
                                LogItem::SetHook(_, prefix, link) => {
                                    if let Some(hook_sender) = &db.hook_sender {
                                        let (tx, mut rx) = channel(10);
                                        let action = HookManagerAction::Set(tx, prefix, link);

                                        hook_sender.send(action).await.unwrap();

                                        let a = rx.recv().await.unwrap();
                                        if a != HookManagerResponse::Ok
                                            && a != HookManagerResponse::Error(
                                                "Already defined".to_string(),
                                            )
                                        {
                                            tracing::error!("failed to set hook: {:?}", a);
                                            return;
                                        }
                                    }
                                }
                                LogItem::RemHook(_, prefix, link) => {
                                    if let Some(hook_sender) = &db.hook_sender {
                                        let (tx, mut rx) = channel(10);
                                        let action = HookManagerAction::Remove(tx, prefix, link);

                                        hook_sender.send(action).await.unwrap();

                                        let a = rx.recv().await.unwrap();
                                        if a != HookManagerResponse::Ok {
                                            tracing::error!("failed to remove hook: {:?}", a);
                                            return;
                                        }
                                    }
                                }
                                LogItem::Push(_, key, value) => {
                                    db.push(KeyType::Record(key), value).await.unwrap();
                                }
                                LogItem::Pop(_, key) => {
                                    db.pop(KeyType::Record(key)).await.unwrap();
                                }
                                _ => (),
                            }
                        }
                    }
                    other => {
                        tracing::error!("invalid return type from logger: {:?}", other);
                        return;
                    }
                },
                None => {
                    tracing::error!("failed to read append file");
                    return;
                }
            }

            tracing::info!("finished to restore data from append file");

            // Turn on hook execution back
            tracing::debug!("turn on hook execution back after data restore");
            if let Some(hook_sender) = &db.hook_sender {
                let (tx, mut rx) = channel(10);
                let action = HookManagerAction::Enable(tx, true);

                hook_sender.send(action).await.unwrap();

                let a = rx.recv().await.unwrap();
                if a != HookManagerResponse::Ok {
                    tracing::error!("failed to turn off hook exectuin: {:?}", a);
                    return;
                }
            }

            tracing::debug!("subscribe to logger");
            db.subscribe_to_logger(sender);
        }

        //
        // Start database process to host on mpsc
        //
        while let Some(data) = rx.recv().await {
            let received_at =
                match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!("Failed to get system timer: {}", e);
                        return;
                    }
                };

            tracing::trace!("received request: {}", data);
            match data {
                // Handle Get actions
                DatabaseAction::Get(sender, key) => {
                    match db.get(KeyType::Record(key.clone())) {
                        Ok(value) => send_response!(sender, Ok(value)),
                        Err(e) => send_response!(sender, Err(e)),
                    }

                    if let Some(sender) = &db.logger_sender {
                        write_log!(sender, vec![LogItem::GetKey(received_at, key)]);
                    }
                }
                // Handle Set actions
                DatabaseAction::Set(sender, key, value) => {
                    match db
                        .insert(
                            KeyType::Record(key.clone()),
                            ValueType::RecordPointer(value.clone()),
                        )
                        .await
                    {
                        Ok(_) => send_response!(sender, Ok(())),
                        Err(e) => send_response!(sender, Err(e)),
                    }

                    if let Some(sender) = &db.logger_sender {
                        write_log!(sender, vec![LogItem::SetKey(received_at, key, value)]);
                    }
                }
                // Handle DeleteKey actions
                DatabaseAction::DeleteKey(sender, key) => {
                    match db.delete_key(KeyType::Record(key.clone())).await {
                        Ok(_) => send_response!(sender, Ok(())),
                        Err(e) => send_response!(sender, Err(e)),
                    }

                    if let Some(sender) = &db.logger_sender {
                        write_log!(sender, vec![LogItem::RemKey(received_at, key)]);
                    }
                }
                // Handle DeleteTable actions
                DatabaseAction::DeleteTable(sender, key) => {
                    match db.delete_table(KeyType::Table(key.clone())).await {
                        Ok(_) => send_response!(sender, Ok(())),
                        Err(e) => send_response!(sender, Err(e)),
                    }

                    if let Some(sender) = &db.logger_sender {
                        write_log!(sender, vec![LogItem::RemPath(received_at, key)]);
                    }
                }
                // Handle ListKeys action
                DatabaseAction::ListKeys(sender, key, level) => {
                    match db.list_keys(KeyType::Record(key.clone()), level) {
                        Ok(list) => send_response!(sender, Ok(list)),
                        Err(e) => send_response!(sender, Err(e)),
                    }

                    if let Some(sender) = &db.logger_sender {
                        write_log!(sender, vec![LogItem::ListKeys(received_at, key)]);
                    }
                }
                // Trigger HookManager
                DatabaseAction::Trigger(sender, key, value) => {
                    match db
                        .trigger(
                            KeyType::Record(key.clone()),
                            ValueType::RecordPointer(value.clone()),
                        )
                        .await
                    {
                        Ok(_) => send_response!(sender, Ok(())),
                        Err(e) => send_response!(sender, Err(e)),
                    }

                    if let Some(sender) = &db.logger_sender {
                        write_log!(sender, vec![LogItem::Trigger(received_at, key, value)]);
                    }
                }
                // Set hook
                DatabaseAction::HookSet(sender, prefix, link) => {
                    match &db.hook_sender {
                        Some(hook_sender) => {
                            let (tx, mut rx) = channel(100);
                            let action = HookManagerAction::Set(tx, prefix.clone(), link.clone());
                            hook_send!(sender, hook_sender, action);

                            match rx.recv().await {
                                Some(response) => match response {
                                    HookManagerResponse::Ok => {
                                        send_response!(sender, Ok(()));
                                    }
                                    _ => send_response!(
                                        sender,
                                        Err(ErrorKind::InternalError(
                                            "Failed to add hook".to_string()
                                        ))
                                    ),
                                },
                                None => hook_receive_failed!(sender, "failed to get answer"),
                            }
                        }
                        None => hook_inactive!(sender),
                    }

                    if let Some(sender) = &db.logger_sender {
                        write_log!(sender, vec![LogItem::SetHook(received_at, prefix, link)]);
                    }
                }
                // Get links for specific hook
                DatabaseAction::HookGet(sender, prefix) => {
                    match &db.hook_sender {
                        Some(hook_sender) => {
                            let (tx, mut rx) = channel(100);
                            let action = HookManagerAction::Get(tx, prefix.clone());
                            hook_send!(sender, hook_sender, action);

                            match rx.recv().await {
                                Some(response) => match response {
                                    HookManagerResponse::Hook(prefix, hooks) => {
                                        send_response!(sender, Ok((prefix, hooks)))
                                    }
                                    _ => send_response!(
                                        sender,
                                        Err(ErrorKind::InvalidKey("Hook is not found".to_string()))
                                    ),
                                },
                                None => hook_receive_failed!(sender, "failed to get answer"),
                            }
                        }
                        None => hook_inactive!(sender),
                    }

                    if let Some(sender) = &db.logger_sender {
                        write_log!(sender, vec![LogItem::GetHook(received_at, prefix)]);
                    }
                }
                // List hooks
                DatabaseAction::HookList(sender, prefix) => {
                    match &db.hook_sender {
                        Some(hook_sender) => {
                            let (tx, mut rx) = channel(100);
                            let action = HookManagerAction::List(tx, prefix.clone());

                            hook_send!(sender, hook_sender, action);

                            match rx.recv().await {
                                Some(response) => match response {
                                    HookManagerResponse::HookList(list) => {
                                        send_response!(sender, Ok(list))
                                    }
                                    _ => send_response!(
                                        sender,
                                        Err(ErrorKind::InvalidKey("Hook is not found".to_string()))
                                    ),
                                },
                                None => hook_receive_failed!(sender, "failed to get answer"),
                            }
                        }
                        None => hook_inactive!(sender),
                    }

                    if let Some(sender) = &db.logger_sender {
                        write_log!(sender, vec![LogItem::ListHooks(received_at, prefix)]);
                    }
                }
                // Remove existing hooks
                DatabaseAction::HookRemove(sender, prefix, link) => {
                    match &db.hook_sender {
                        Some(hook_sender) => {
                            let (tx, mut rx) = channel(100);
                            let action =
                                HookManagerAction::Remove(tx, prefix.clone(), link.clone());

                            hook_send!(sender, hook_sender, action);

                            match rx.recv().await {
                                Some(response) => match response {
                                    HookManagerResponse::Ok => {
                                        send_response!(sender, Ok(()));
                                    }
                                    _ => send_response!(
                                        sender,
                                        Err(ErrorKind::InvalidKey("Hook is not found".to_string()))
                                    ),
                                },
                                None => hook_receive_failed!(sender, "failed to get answer"),
                            }
                        }
                        None => hook_inactive!(sender),
                    }

                    if let Some(sender) = &db.logger_sender {
                        write_log!(sender, vec![LogItem::RemHook(received_at, prefix, link)]);
                    }
                }
                // Resume logging
                DatabaseAction::ResumeLog(sender) => {
                    if let Some(logger_sender) = &db.logger_sender {
                        let (tx, mut rx) = channel(100);
                        send_response_with_mutex_sender!(logger_sender, LoggerAction::Resume(tx));

                        match rx.recv().await {
                            Some(response) => match response {
                                LoggerResponse::Ok => send_response!(sender, Ok(())),
                                LoggerResponse::Err(e) => {
                                    send_response!(sender, Err(ErrorKind::LogError(e)))
                                }
                                other => {
                                    tracing::error!("logger should have return Ok or Err but returned with: {:?}", other);
                                    send_response!(
                                        sender,
                                        Err(ErrorKind::LogError("internal error".to_string()))
                                    );
                                }
                            },
                            None => {
                                tracing::error!("failed to get answer");
                                send_response!(
                                    sender,
                                    Err(ErrorKind::LogError("failed to get answer".to_string()))
                                );
                            }
                        }
                    }
                }
                // Suspend logging
                DatabaseAction::SuspendLog(sender) => {
                    if let Some(logger_sender) = &db.logger_sender {
                        let (tx, mut rx) = channel(100);
                        send_response_with_mutex_sender!(logger_sender, LoggerAction::Suspend(tx));

                        match rx.recv().await {
                            Some(response) => match response {
                                LoggerResponse::Ok => send_response!(sender, Ok(())),
                                LoggerResponse::Err(e) => {
                                    send_response!(sender, Err(ErrorKind::LogError(e)))
                                }
                                other => {
                                    tracing::error!("logger should have return Ok or Err but returned with: {:?}", other);
                                    send_response!(
                                        sender,
                                        Err(ErrorKind::LogError("internal error".to_string()))
                                    );
                                }
                            },
                            None => {
                                tracing::error!("failed to get answer");
                                send_response!(
                                    sender,
                                    Err(ErrorKind::LogError("failed to get answer".to_string()))
                                );
                            }
                        }
                    }
                }
                // Push to a queue
                DatabaseAction::Push(sender, key, value) => {
                    match db.push(KeyType::Record(key.clone()), value.clone()).await {
                        Ok(_) => send_response!(sender, Ok(())),
                        Err(e) => send_response!(sender, Err(e)),
                    }

                    if let Some(sender) = &db.logger_sender {
                        write_log!(sender, vec![LogItem::Push(received_at, key, value)]);
                    }
                }
                // Pop from queue
                DatabaseAction::Pop(sender, key) => {
                    match db.pop(KeyType::Record(key.clone())).await {
                        Ok(value) => send_response!(sender, Ok(ValueType::RecordPointer(value))),
                        Err(e) => send_response!(sender, Err(e)),
                    }

                    if let Some(sender) = &db.logger_sender {
                        write_log!(sender, vec![LogItem::Pop(received_at, key)]);
                    }
                }
            }
        }
    });

    (tx, thread)
}

macro_rules! hook_inactive {
    ($sender:expr) => {
        $sender
            .send(Err(ErrorKind::InactiveHookManager))
            .await
            .unwrap_or_else(|e| tracing::error!("Error during send: {}", e))
    };
}
pub(self) use hook_inactive;

macro_rules! hook_send {
    ($sender:expr, $hook_sender:expr, $action:expr) => {
        if let Err(e) = $hook_sender.send($action).await {
            tracing::error!("Failed to send to hook manager: {}", e);
            $sender
                .send(Err(ErrorKind::InternalError("".to_string())))
                .await
                .unwrap_or_else(|e| tracing::error!("Error during send: {}", e));
            continue;
        }
    };
}
pub(self) use hook_send;

macro_rules! hook_receive_failed {
    ($sender:expr, $error:expr) => {{
        tracing::error!("Failed to receive from hook manager: {}", $error);
        $sender
            .send(Err(ErrorKind::InternalError(
                "Failed to receive from hook manager".to_string(),
            )))
            .await
            .unwrap_or_else(|e| tracing::error!("Error during send: {}", e));
    }};
}
pub(self) use hook_receive_failed;

macro_rules! send_response {
    ($sender:expr, $value:expr) => {{
        $sender
            .send($value)
            .await
            .unwrap_or_else(|e| tracing::error!("Error during send: {}", e));
    }};
}
pub(self) use send_response;

macro_rules! send_response_with_mutex_sender {
    ($sender:expr, $value:expr) => {{
        $sender
            .send($value)
            .await
            .unwrap_or_else(|e| tracing::error!("Error during send: {}", e));
    }};
}
pub(self) use send_response_with_mutex_sender;

macro_rules! write_log {
    ($logger_sender:expr, $messages:expr) => {
        $logger_sender
            .send(LoggerAction::WriteAsync($messages))
            .await
            .unwrap_or_else(|e| tracing::error!("{}", e));
    };
}
pub(self) use write_log;
