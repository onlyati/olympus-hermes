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
use onlyati_datastore::datastore::enums::ListType;
use std::borrow::Cow;
use std::net::SocketAddr;
use std::sync::mpsc::channel;
use std::sync::RwLock;
use std::sync::{mpsc::Sender, Arc, Mutex};
use tower_http::trace::DefaultMakeSpan;
use tower_http::trace::TraceLayer;

// Internal depencies
use super::macros::{send_data_back, send_data_request, verify_one_item, verify_two_items};
use super::structs::{CommandMethod, WsRequest, WsResponse};
use crate::server::utilities::config_parse::Config;
use onlyati_datastore::datastore::{enums::pair::ValueType, enums::DatabaseAction};

/// Struct that is injected into every endpoint
#[derive(Clone)]
pub struct InjectedData {
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
    config: Arc<RwLock<Config>>,
}

async fn handle_request(req: WsRequest, injected: &InjectedData) -> WsResponse {
    match req.command {
        CommandMethod::GetKey => {
            let key = verify_one_item!(req.key, "'key' must be specified");

            let (tx, rx) = channel();
            let action = DatabaseAction::Get(tx, key);
            send_data_request!(action, injected.data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(value) => match value {
                        ValueType::RecordPointer(data) => return WsResponse::new_ok(data),
                        _ => return WsResponse::new_err("Pointer must be Record but it was Table"),
                    },
                    Err(e) => return WsResponse::new_err(e.to_string()),
                },
                Err(e) => {
                    for line in e.to_string().lines() {
                        tracing::error!("{}", line);
                    }
                    return WsResponse::new_err("internal server error");
                }
            }
        }
        CommandMethod::SetKey => {
            let (key, value) = verify_two_items!(req.key, req.value, "'key' and 'value' must be specified");

            let (tx, rx) = channel();
            let action = DatabaseAction::Set(tx, key, value);
            send_data_request!(action, injected.data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(_) => return WsResponse::new_ok(""),
                    Err(e) => return WsResponse::new_err(e),
                },
                Err(e) => {
                    for line in e.to_string().lines() {
                        tracing::error!("{}", line);
                    }
                    return WsResponse::new_err("internal server error");
                }
            }
        }
        CommandMethod::RemKey => {
            let key = verify_one_item!(req.key, "'key' must be specified");

            let (tx, rx) = channel();
            let action = DatabaseAction::DeleteKey(tx, key);
            send_data_request!(action, injected.data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(_) => return WsResponse::new_ok(""),
                    Err(e) => return WsResponse::new_err(e),
                },
                Err(e) => {
                    for line in e.to_string().lines() {
                        tracing::error!("{}", line);
                    }
                    return WsResponse::new_err("internal server error");
                }
            }
        }
        CommandMethod::RemPath => {
            let key = verify_one_item!(req.key, "'key' must be specified");

            let (tx, rx) = channel();
            let action = DatabaseAction::DeleteTable(tx, key);
            send_data_request!(action, injected.data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(_) => return WsResponse::new_ok(""),
                    Err(e) => return WsResponse::new_err(e),
                },
                Err(e) => {
                    for line in e.to_string().lines() {
                        tracing::error!("{}", line);
                    }
                    return WsResponse::new_err("internal server error");
                }
            }
        }
        CommandMethod::ListKeys => {
            let key = verify_one_item!(req.key, "'key' must be specified");

            let (tx, rx) = channel();
            let action = DatabaseAction::ListKeys(tx, key, ListType::All);
            send_data_request!(action, injected.data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(list) => {
                        let mut data = String::new();
                        for key in list {
                            data += key.get_key();
                            data += "\n";
                        }
                        return WsResponse::new_ok(data);
                    }
                    Err(e) => return WsResponse::new_err(e),
                },
                Err(e) => {
                    for line in e.to_string().lines() {
                        tracing::error!("{}", line);
                    }
                    return WsResponse::new_err("internal server error");
                }
            }
        }
        CommandMethod::Trigger => {
            let (key, value) = verify_two_items!(req.key, req.value, "'key' and 'value' must be specified");

            let (tx, rx) = channel();
            let action = DatabaseAction::Trigger(tx, key, value);
            send_data_request!(action, injected.data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(_) => return WsResponse::new_ok(""),
                    Err(e) => return WsResponse::new_err(e),
                },
                Err(e) => {
                    for line in e.to_string().lines() {
                        tracing::error!("{}", line);
                    }
                    return WsResponse::new_err("internal server error");
                }
            }
        }
        CommandMethod::GetHook => {
            let prefix = verify_one_item!(req.prefix, "'prefix' must be specified");

            let (tx, rx) = channel();
            let action = DatabaseAction::HookGet(tx, prefix);
            send_data_request!(action, injected.data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok((_prefix, links)) => {
                        let mut response = String::new();
                        for link in links {
                            response += &link[..];
                            response += "\n";
                        }
                        return WsResponse::new_ok(response);
                    }
                    Err(e) => return WsResponse::new_err(e),
                },
                Err(e) => {
                    for line in e.to_string().lines() {
                        tracing::error!("{}", line);
                    }
                    return WsResponse::new_err("internal server error");
                }
            }
        },
        CommandMethod::SetHook => {
            let (prefix, link) = verify_two_items!(req.prefix, req.link, "'prefix' and 'link' must be specified");

            let (tx, rx) = channel();
            let action = DatabaseAction::HookSet(tx, prefix, link);
            send_data_request!(action, injected.data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(_) => return WsResponse::new_ok(""),
                    Err(e) => return WsResponse::new_err(e),
                },
                Err(e) => {
                    for line in e.to_string().lines() {
                        tracing::error!("{}", line);
                    }
                    return WsResponse::new_err("internal server error");
                }
            }
        },
        CommandMethod::RemHook => {
            let (prefix, link) = verify_two_items!(req.prefix, req.link, "'prefix' and 'link' must be specified");

            let (tx, rx) = channel();
            let action = DatabaseAction::HookRemove(tx, prefix, link);
            send_data_request!(action, injected.data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(_) => return WsResponse::new_ok(""),
                    Err(e) => return WsResponse::new_err(e),
                },
                Err(e) => {
                    for line in e.to_string().lines() {
                        tracing::error!("{}", line);
                    }
                    return WsResponse::new_err("internal server error");
                }
            }
        },
        CommandMethod::ListHooks => {
            let prefix = verify_one_item!(req.prefix, "'prefix' must be specified");

            let (tx, rx) = channel();
            let action = DatabaseAction::HookList(tx, prefix);
            send_data_request!(action, injected.data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(hooks) => {
                        let mut response = String::new();
                        for (prefix, links) in hooks {
                            response += format!("{} {:?}\n", prefix, links).as_str();
                        }
                        return WsResponse::new_ok(response);
                    },
                    Err(e) => return WsResponse::new_err(e),
                },
                Err(e) => {
                    for line in e.to_string().lines() {
                        tracing::error!("{}", line);
                    }
                    return WsResponse::new_err("internal server error");
                }
            }
        },
        CommandMethod::SuspendLog => {
            let (tx, rx) = channel();
            let action = DatabaseAction::SuspendLog(tx);
            send_data_request!(action, injected.data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(_) => return WsResponse::new_ok(""),
                    Err(e) => return WsResponse::new_err(e),
                },
                Err(e) => {
                    for line in e.to_string().lines() {
                        tracing::error!("{}", line);
                    }
                    return WsResponse::new_err("internal server error");
                }
            }
        },
        CommandMethod::ResumeLog => {
            let (tx, rx) = channel();
            let action = DatabaseAction::ResumeLog(tx);
            send_data_request!(action, injected.data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(_) => return WsResponse::new_ok(""),
                    Err(e) => return WsResponse::new_err(e),
                },
                Err(e) => {
                    for line in e.to_string().lines() {
                        tracing::error!("{}", line);
                    }
                    return WsResponse::new_err("internal server error");
                }
            }
        },
        CommandMethod::Exec => unimplemented!(),
        CommandMethod::Push => {
            let (key, value) = verify_two_items!(req.key, req.value, "'key' and 'value' must be specified");

            let (tx, rx) = channel();
            let action = DatabaseAction::Push(tx, key, value);
            send_data_request!(action, injected.data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(_) => return WsResponse::new_ok(""),
                    Err(e) => return WsResponse::new_err(e),
                },
                Err(e) => {
                    for line in e.to_string().lines() {
                        tracing::error!("{}", line);
                    }
                    return WsResponse::new_err("internal server error");
                }
            }
        }
        CommandMethod::Pop => {
            let key = verify_one_item!(req.key, "'key' must be specified");

            let (tx, rx) = channel();
            let action = DatabaseAction::Pop(tx, key);
            send_data_request!(action, injected.data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(value) => match value {
                        ValueType::RecordPointer(data) => return WsResponse::new_ok(data),
                        _ => return WsResponse::new_err("Pointer must be Record but it was Table"),
                    },
                    Err(e) => return WsResponse::new_err(e.to_string()),
                },
                Err(e) => {
                    for line in e.to_string().lines() {
                        tracing::error!("{}", line);
                    }
                    return WsResponse::new_err("internal server error");
                }
            }
        }
    }

    return WsResponse::new_err("unimplemented response");
}

/// Handle the requests coming via websocket
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

/// Last point before connection would be upgraded to ws
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

    return ws.on_upgrade(move |socket| handle_socket(socket, addr, injected));
}

/// Start the websocket server
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
