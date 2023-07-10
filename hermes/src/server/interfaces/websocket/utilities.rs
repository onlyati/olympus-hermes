use axum::extract::State;
// External depencies
use axum::extract::ws::{CloseFrame, Message};
use axum::TypedHeader;
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        ConnectInfo,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use std::borrow::Cow;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc::channel, mpsc::Sender, Mutex, RwLock};
use tower_http::trace::DefaultMakeSpan;
use tower_http::trace::TraceLayer;

// Internal depencies
use super::macros::{send_data_back, send_data_request, verify_one_item, verify_two_items};
use crate::common::websocket::{CommandMethod, WsRequest, WsResponse};
use crate::server::utilities::config_parse::Config;
use onlyati_datastore::datastore::{
    enums::pair::ValueType, enums::DatabaseAction, enums::ListType,
};

/// Struct that is injected into every endpoint
#[derive(Clone)]
pub struct InjectedData {
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
    config: Arc<RwLock<Config>>,
}

/// Handle the request that is coming via websocket calls
///
/// # Parameters
/// - `req`: Request itself
/// - `injected`: This is the state from axum that contains the config and sender for database thread
///
/// # Details
///
/// This is called from `handle_socket` method. This function performs every single action that can be called.
///
/// # Return
///
/// Return with a `WsResponse` structure.
async fn handle_request(req: WsRequest, injected: &InjectedData) -> WsResponse {
    match req.command {
        //
        // Get key
        //
        CommandMethod::GetKey => {
            let key = verify_one_item!(req.key, "'key' must be specified");

            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::Get(tx, key);
            send_data_request!(action, injected.data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(value) => match value {
                        ValueType::RecordPointer(data) => WsResponse::new_ok(data),
                        _ => WsResponse::new_err("Pointer must be Record but it was Table"),
                    },
                    Err(e) => WsResponse::new_err(e.to_string()),
                },
                None => {
                    tracing::error!("failed to receive from database");
                    WsResponse::new_err("internal server error")
                }
            }
        }
        //
        // Set key
        //
        CommandMethod::SetKey => {
            let (key, value) =
                verify_two_items!(req.key, req.value, "'key' and 'value' must be specified");

            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::Set(tx, key, value);
            send_data_request!(action, injected.data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(_) => WsResponse::new_ok(""),
                    Err(e) => WsResponse::new_err(e),
                },
                None => {
                    tracing::error!("failed to receive from database");
                    WsResponse::new_err("internal server error")
                }
            }
        }
        //
        // Remove key
        //
        CommandMethod::RemKey => {
            let key = verify_one_item!(req.key, "'key' must be specified");

            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::DeleteKey(tx, key);
            send_data_request!(action, injected.data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(_) => WsResponse::new_ok(""),
                    Err(e) => WsResponse::new_err(e),
                },
                None => {
                    tracing::error!("failed to receive from database");
                    WsResponse::new_err("internal server error")
                }
            }
        }
        //
        // Remove path
        //
        CommandMethod::RemPath => {
            let key = verify_one_item!(req.key, "'key' must be specified");

            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::DeleteTable(tx, key);
            send_data_request!(action, injected.data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(_) => WsResponse::new_ok(""),
                    Err(e) => WsResponse::new_err(e),
                },
                None => {
                    tracing::error!("failed to receive from database");
                    WsResponse::new_err("internal server error")
                }
            }
        }
        //
        // List keys
        //
        CommandMethod::ListKeys => {
            let key = verify_one_item!(req.key, "'key' must be specified");

            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::ListKeys(tx, key, ListType::All);
            send_data_request!(action, injected.data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(list) => {
                        let mut data = String::new();
                        for key in list {
                            data += key.get_key();
                            data += "\n";
                        }
                        WsResponse::new_ok(data)
                    }
                    Err(e) => WsResponse::new_err(e),
                },
                None => {
                    tracing::error!("failed to receive from database");
                    WsResponse::new_err("internal server error")
                }
            }
        }
        //
        // Send trigger
        //
        CommandMethod::Trigger => {
            let (key, value) =
                verify_two_items!(req.key, req.value, "'key' and 'value' must be specified");

            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::Trigger(tx, key, value);
            send_data_request!(action, injected.data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(_) => WsResponse::new_ok(""),
                    Err(e) => WsResponse::new_err(e),
                },
                None => {
                    tracing::error!("failed to receive from database");
                    WsResponse::new_err("internal server error")
                }
            }
        }
        //
        // Get hook
        //
        CommandMethod::GetHook => {
            let prefix = verify_one_item!(req.prefix, "'prefix' must be specified");

            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::HookGet(tx, prefix);
            send_data_request!(action, injected.data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok((_prefix, links)) => {
                        let mut response = String::new();
                        for link in links {
                            response += &link[..];
                            response += "\n";
                        }
                        WsResponse::new_ok(response)
                    }
                    Err(e) => WsResponse::new_err(e),
                },
                None => {
                    tracing::error!("failed to receive from database");
                    WsResponse::new_err("internal server error")
                }
            }
        }
        //
        // Set hook
        //
        CommandMethod::SetHook => {
            let (prefix, link) = verify_two_items!(
                req.prefix,
                req.link,
                "'prefix' and 'link' must be specified"
            );

            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::HookSet(tx, prefix, link);
            send_data_request!(action, injected.data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(_) => WsResponse::new_ok(""),
                    Err(e) => WsResponse::new_err(e),
                },
                None => {
                    tracing::error!("failed to receive from database");
                    WsResponse::new_err("internal server error")
                }
            }
        }
        //
        // Remove hook
        //
        CommandMethod::RemHook => {
            let (prefix, link) = verify_two_items!(
                req.prefix,
                req.link,
                "'prefix' and 'link' must be specified"
            );

            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::HookRemove(tx, prefix, link);
            send_data_request!(action, injected.data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(_) => WsResponse::new_ok(""),
                    Err(e) => WsResponse::new_err(e),
                },
                None => {
                    tracing::error!("failed to receive from database");
                    WsResponse::new_err("internal server error")
                }
            }
        }
        //
        // List hooks
        //
        CommandMethod::ListHooks => {
            let prefix = verify_one_item!(req.prefix, "'prefix' must be specified");

            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::HookList(tx, prefix);
            send_data_request!(action, injected.data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(hooks) => {
                        let mut response = String::new();
                        for (prefix, links) in hooks {
                            response += format!("{} {:?}\n", prefix, links).as_str();
                        }
                        WsResponse::new_ok(response)
                    }
                    Err(e) => WsResponse::new_err(e),
                },
                None => {
                    tracing::error!("failed to receive from database");
                    WsResponse::new_err("internal server error")
                }
            }
        }
        //
        // Suspend log
        //
        CommandMethod::SuspendLog => {
            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::SuspendLog(tx);
            send_data_request!(action, injected.data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(_) => WsResponse::new_ok(""),
                    Err(e) => WsResponse::new_err(e),
                },
                None => {
                    tracing::error!("failed to receive from database");
                    WsResponse::new_err("internal server error")
                }
            }
        }
        //
        // Resume log
        //
        CommandMethod::ResumeLog => {
            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::ResumeLog(tx);
            send_data_request!(action, injected.data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(_) => WsResponse::new_ok(""),
                    Err(e) => WsResponse::new_err(e),
                },
                None => {
                    tracing::error!("failed to receive from database");
                    WsResponse::new_err("internal server error")
                }
            }
        }
        //
        // Execute lua script
        //
        CommandMethod::Exec => {
            let (script, save) =
                verify_two_items!(req.exec, req.save, "'exec' and 'save' must be specified");
            let (key, value) =
                verify_two_items!(req.key, req.value, "'key' and 'value' must be specified");

            // Get the old value of the keys
            let (tx, mut rx) = channel(10);
            let get_action = DatabaseAction::Get(tx, key.clone());

            send_data_request!(get_action, injected.data_sender);

            let old_pair = match rx.recv().await {
                Some(response) => match response {
                    Ok(value) => match value {
                        ValueType::RecordPointer(data) => Some((key.clone(), data)),
                        _ => {
                            tracing::error!("Pointer must be Record but it was Table");
                            return WsResponse::new_err("internal server error");
                        }
                    },
                    Err(_) => None,
                },
                None => {
                    tracing::error!("failed to receive from database");
                    return WsResponse::new_err("internal server error");
                }
            };

            // Get config
            let config = {
                let config = injected.config.read().await;
                match &config.scripts {
                    Some(scr) => match scr.execs.contains(&script) {
                        true => scr.clone(),
                        false => return WsResponse::new_err("requested script is not defined"),
                    },
                    None => return WsResponse::new_err("requested script is not defined"),
                }
            };

            let new_pair = (key.clone(), value.clone());

            // Call lua utility
            let modified_pair = match crate::server::utilities::lua::run(
                config, old_pair, new_pair, script, req.parm,
            )
            .await
            {
                Ok(modified_pair) => modified_pair,
                Err(e) => {
                    for line in e.lines() {
                        tracing::error!("{}", line);
                    }
                    return WsResponse::new_err("failed to execute script".to_string());
                }
            };

            // Make a SET action for the modified pair
            if save {
                if modified_pair.1.is_empty() {
                    tracing::debug!("value is empty so delete the key");
                    let (tx, mut rx) = channel(10);

                    let action = DatabaseAction::DeleteKey(tx, modified_pair.0);
                    send_data_request!(action, injected.data_sender);

                    match rx.recv().await {
                        Some(response) => match response {
                            Ok(_) => {
                                tracing::debug!("key successufully deleted");
                                WsResponse::new_ok("")
                            }
                            Err(e) => {
                                tracing::debug!("failed to delete the key: {}", e);
                                WsResponse::new_err(e)
                            }
                        },
                        None => {
                            tracing::error!("failed to receive from database");
                            WsResponse::new_err("internal server error")
                        }
                    }
                } else {
                    tracing::debug!("value is specified so set the key");
                    let (tx, mut rx) = channel(10);
                    let action = DatabaseAction::Set(tx, modified_pair.0, modified_pair.1);
                    send_data_request!(action, injected.data_sender);

                    match rx.recv().await {
                        Some(response) => match response {
                            Ok(_) => {
                                tracing::debug!("key successfully saved");
                                WsResponse::new_ok("")
                            }
                            Err(e) => {
                                tracing::debug!("failed to save key: {}", e);
                                WsResponse::new_err(e)
                            }
                        },
                        None => {
                            tracing::error!("failed to receive from database");
                            WsResponse::new_err("internal server error")
                        }
                    }
                }
            }
            // Or a TRIGGER if this was requested
            else if !modified_pair.1.is_empty() {
                tracing::debug!("it is just a trigger");
                let (tx, mut rx) = channel(10);
                let action = DatabaseAction::Trigger(tx, modified_pair.0, modified_pair.1);
                send_data_request!(action, injected.data_sender);

                match rx.recv().await {
                    Some(response) => match response {
                        Ok(_) => {
                            tracing::debug!("trigger successfully set");
                            WsResponse::new_ok("")
                        }
                        Err(e) => {
                            tracing::debug!("failed to set trigger: {}", e);
                            WsResponse::new_err(e)
                        }
                    },
                    None => {
                        tracing::error!("failed to receive from database");
                        WsResponse::new_err("internal server error")
                    }
                }
            } else {
                WsResponse::new_err("After script was run, the new value is empty")
            }
        }
        //
        // Push pair to a queue
        //
        CommandMethod::Push => {
            let (key, value) =
                verify_two_items!(req.key, req.value, "'key' and 'value' must be specified");

            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::Push(tx, key, value);
            send_data_request!(action, injected.data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(_) => WsResponse::new_ok(""),
                    Err(e) => WsResponse::new_err(e),
                },
                None => {
                    tracing::error!("failed to receive from database");
                    WsResponse::new_err("internal server error")
                }
            }
        }
        //
        // Pop pair from the queue
        //
        CommandMethod::Pop => {
            let key = verify_one_item!(req.key, "'key' must be specified");

            let (tx, mut rx) = channel(10);
            let action = DatabaseAction::Pop(tx, key);
            send_data_request!(action, injected.data_sender);

            match rx.recv().await {
                Some(response) => match response {
                    Ok(value) => match value {
                        ValueType::RecordPointer(data) => WsResponse::new_ok(data),
                        _ => WsResponse::new_err("Pointer must be Record but it was Table"),
                    },
                    Err(e) => WsResponse::new_err(e.to_string()),
                },
                None => {
                    tracing::error!("failed to receive from database");
                    WsResponse::new_err("internal server error")
                }
            }
        }
    }
}

/// Handle the requests coming via websocket
///
/// # Parameters
/// - `socket`: websocker that is used for receive and send data
/// - `who`: address where the request was caming from
/// - `injected`: state from axum
///
/// # Details
///
/// This function is called when the /ws endpoint is called. This is responsible for the websocket communication.
/// This function does:
/// - Process the command if incoming message was `Message::Text` request. Text is passed to `handle_request` function.
/// - Respond with a `Message::Pong` for a `Message::Ping`
/// - Gracefully shotdown the communication for `Message::Clonse` request
async fn handle_socket(mut socket: WebSocket, who: SocketAddr, injected: InjectedData) {
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(msg) => match msg {
                // Regular request
                Message::Text(text) => {
                    tracing::trace!("received: {}", text);

                    let request = match WsRequest::from(&text[..]) {
                        Ok(req) => req,
                        Err(e) => {
                            send_data_back!(socket, Message::Text(e));
                            continue;
                        }
                    };

                    let response = handle_request(request, &injected).await;
                    match serde_json::to_string(&response) {
                        Ok(str) => send_data_back!(socket, Message::Text(str)),
                        Err(e) => {
                            for line in e.to_string().lines() {
                                tracing::error!("{}", line);
                            }
                            send_data_back!(
                                socket,
                                Message::Close(Some(CloseFrame {
                                    code: 1003,
                                    reason: Cow::from("internal server error")
                                }))
                            );
                            return;
                        }
                    }
                }
                // Close connection normally
                Message::Close(c) => {
                    if let Some(cf) = c {
                        tracing::debug!(
                            "{} sent close with code {} and reason {}",
                            who,
                            cf.code,
                            cf.reason
                        );
                        return;
                    }
                }
                // Receive a ping, send a pong back
                Message::Ping(v) => {
                    tracing::debug!("received a ping {:?}", v);
                    send_data_back!(socket, Message::Pong(v));
                }
                other => {
                    tracing::warn!("not handled type: {:?}", other);
                    send_data_back!(
                        socket,
                        Message::Close(Some(CloseFrame {
                            code: 1003,
                            reason: Cow::from("invalid type")
                        }))
                    );
                    return;
                }
            },
            Err(e) => {
                tracing::error!("failed to receive message: {}", e);
                return;
            }
        }
    }
}

/// Endpoint for /ws URI
///
/// # Parameters
/// - `injected`: state from axum
/// - `ws`: used to upgrade connection to websocket
/// - `user_agent`: where from was it called
/// - `addr`: connection information
///
/// # Details
///
/// This is called for GET /ws request. This is the last point before it would be upgrade to websocket,
/// so this is the last action to gather connection information.
async fn ws_handler(
    State(injected): State<InjectedData>,
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    tracing::debug!("`{user_agent}` at {addr} connected");

    ws.on_upgrade(move |socket| handle_socket(socket, addr, injected))
}

/// Start the websocket server
///
/// # Parameters
/// - `data_sender`: Sender to send data to database thread
/// - `address`: where it should listen
/// - `config`: application configuration
///
/// # Details
///
/// This is called to run this interface. `data_sender` and `config` will be shared in endpoints.
pub async fn run_async(
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
    address: String,
    config: Arc<RwLock<Config>>,
) {
    tracing::info!("Websocket interface on {} is starting...", address);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(InjectedData {
            data_sender,
            config,
        });

    let address: SocketAddr = match address.parse() {
        Ok(addr) => addr,
        Err(e) => {
            for line in e.to_string().lines() {
                tracing::error!("{}", line);
            }
            return;
        }
    };

    if let Err(e) = axum::Server::bind(&address)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
    {
        for line in e.to_string().lines() {
            tracing::error!("{}", line);
        }
    }
}
