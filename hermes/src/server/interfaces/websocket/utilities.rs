// External depencies
use axum::extract::ws::Message;
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
use std::net::SocketAddr;
use std::sync::RwLock;
use std::sync::{mpsc::Sender, Arc, Mutex};
use tower_http::trace::DefaultMakeSpan;
use tower_http::trace::TraceLayer;

// Internal depencies
use super::macros::send_data_back;
use super::structs::WsRequest;
use crate::server::utilities::config_parse::Config;
use onlyati_datastore::datastore::enums::DatabaseAction;

/// Struct that is injected into every endpoint
#[derive(Clone)]
pub struct InjectedData {
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
    config: Arc<RwLock<Config>>,
}

/// Handle the requests coming via websocket
async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(msg) => match msg {
                // Regular request
                Message::Text(text) => {
                    tracing::trace!("received: {}", text);

                    let request = match WsRequest::from(&text[..]) {
                        Ok(req) => req,
                        Err(e) => {
                            tracing::error!("failed to parse request");
                            for line in e.lines() {
                                tracing::error!("{}", line);
                            }
                            send_data_back!(socket, Message::Text(e));
                            return;
                        }
                    };

                    send_data_back!(
                        socket,
                        Message::Text(serde_json::to_string(&request).unwrap())
                    );
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
                    send_data_back!(socket, Message::Text("Not handled data type".to_string()));
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
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    tracing::info!("`{user_agent}` at {addr} connected");

    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

/// Start the websocket server
pub async fn run_async(
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
    address: String,
    config: Arc<RwLock<Config>>,
) {
    tracing::info!("REST interface on {} is starting...", address);

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
