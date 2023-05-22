// External depencies
use axum::error_handling::HandleErrorLayer;
use axum::BoxError;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::mpsc::channel;
use std::sync::{mpsc::Sender, Arc, Mutex};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

// Internal depencies
use onlyati_datastore::datastore::{enums::pair::ValueType, enums::DatabaseAction, utilities};
use onlyati_datastore::hook::enums::{HookManagerAction, HookManagerResponse};
use onlyati_datastore::logger::enums::{LoggerAction, LoggerResponse};

// Import macroes
use super::macros::{
    return_client_error, return_ok, return_ok_with_value, return_server_error, send_data_request,
};

/// Struct that is injected into every endpoint
#[derive(Clone)]
pub struct InjectedData {
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
    hook_sender: Arc<Mutex<Sender<HookManagerAction>>>,
    logger_sender: Option<Arc<Mutex<Sender<LoggerAction>>>>,
}

/// Struct is used to query the SET endpoint
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Pair {
    key: String,
    value: String,
}

/// Struct is used to query the GET and LIST endpoint
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KeyParm {
    key: String,
}

/// Struct is used to query the REMKEY and REMPATH endpoints
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeleteParm {
    key: String,
    kind: Option<String>,
}

/// Struct that is used to return with hook value
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Hook {
    prefix: String,
    links: Vec<String>,
}

/// GET endpoint
async fn get_key(
    State(injected): State<InjectedData>,
    Query(parms): Query<KeyParm>,
) -> impl IntoResponse {
    let (tx, rx) = utilities::get_channel_for_get();
    let get_action = DatabaseAction::Get(tx, parms.key);

    send_data_request!(get_action, injected.data_sender);

    match rx.recv() {
        Ok(response) => match response {
            Ok(value) => match value {
                ValueType::RecordPointer(data) => return_ok_with_value!(data),
                _ => return_server_error!("Pointer must be Record but it was Table"),
            },
            Err(e) => return_client_error!(e.to_string()),
        },
        Err(e) => return_server_error!(e),
    }
}

/// SET endpoint
async fn set_key(
    State(injected): State<InjectedData>,
    Json(pair): Json<Pair>,
) -> impl IntoResponse {
    let (tx, rx) = utilities::get_channel_for_set();
    let set_action = DatabaseAction::Set(tx, pair.key.clone(), pair.value.clone());

    send_data_request!(set_action, injected.data_sender);

    match rx.recv() {
        Ok(response) => match response {
            Ok(_) => return_ok!(),
            Err(e) => return_client_error!(e.to_string()),
        },
        Err(e) => return_server_error!(e),
    }
}

/// REMKEY and REMPATH endpoint
async fn delete_key(
    State(injected): State<InjectedData>,
    Query(parms): Query<DeleteParm>,
) -> impl IntoResponse {
    let (tx, rx) = utilities::get_channel_for_delete();

    let action = match parms.kind {
        Some(kind) => match kind.as_str() {
            "record" => DatabaseAction::DeleteKey(tx, parms.key),
            "path" => DatabaseAction::DeleteTable(tx, parms.key),
            _ => return_client_error!(format!(
                "Only record or path can be delete but not {}",
                kind
            )),
        },
        None => DatabaseAction::DeleteKey(tx, parms.key),
    };

    send_data_request!(action, injected.data_sender);

    match rx.recv() {
        Ok(response) => match response {
            Ok(_) => return_ok!(),
            Err(e) => return_client_error!(e.to_string()),
        },
        Err(e) => return_server_error!(e),
    }
}

/// LIST endpoint
async fn list_keys(
    State(injected): State<InjectedData>,
    Query(parms): Query<KeyParm>,
) -> impl IntoResponse {
    let (tx, rx) = utilities::get_channel_for_list();
    let list_action = DatabaseAction::ListKeys(
        tx,
        parms.key,
        onlyati_datastore::datastore::enums::ListType::All,
    );

    send_data_request!(list_action, injected.data_sender);

    match rx.recv() {
        Ok(response) => match response {
            Ok(list) => return_ok_with_value!(list
                .iter()
                .map(|x| x.get_key().to_string())
                .collect::<Vec<String>>()),
            Err(e) => return_client_error!(e.to_string()),
        },
        Err(e) => return_server_error!(e),
    }
}

/// SET hook
async fn set_hook(
    State(injected): State<InjectedData>,
    Json(pair): Json<Pair>,
) -> impl IntoResponse {
    let (tx, rx) = channel();
    let action = HookManagerAction::Set(tx, pair.key, pair.value);
    send_data_request!(action, injected.hook_sender);

    match rx.recv() {
        Ok(response) => match response {
            HookManagerResponse::Ok => return_ok!(),
            HookManagerResponse::Error(e) => return_client_error!(e),
            _ => return_server_error!("this should happen request should have return Ok or Error"),
        },
        Err(e) => return_server_error!(e),
    }
}

/// GET hook
async fn get_hook(
    State(injected): State<InjectedData>,
    Query(key): Query<KeyParm>,
) -> impl IntoResponse {
    let (tx, rx) = channel();
    let action = HookManagerAction::Get(tx, key.key);
    send_data_request!(action, injected.hook_sender);

    match rx.recv() {
        Ok(response) => match response {
            HookManagerResponse::Hook(prefix, links) => {
                return_ok_with_value!(Hook {
                    prefix: prefix,
                    links: links
                });
            }
            HookManagerResponse::Error(e) => return_client_error!(e),
            _ => {
                return_server_error!("this should happen request should have return Hook or Error")
            }
        },
        Err(e) => return_server_error!(e),
    }
}

/// REMOVE hook
async fn delete_hook(
    State(injected): State<InjectedData>,
    Query(pair): Query<Pair>,
) -> impl IntoResponse {
    let (tx, rx) = channel();
    let action = HookManagerAction::Remove(tx, pair.key, pair.value);
    send_data_request!(action, injected.hook_sender);

    match rx.recv() {
        Ok(response) => match response {
            HookManagerResponse::Ok => return_ok!(),
            HookManagerResponse::Error(e) => return_client_error!(e),
            _ => return_server_error!("this should happen request should have return Ok or Error"),
        },
        Err(e) => return_server_error!(e),
    }
}

/// LIST hooks
async fn list_hooks(
    State(injected): State<InjectedData>,
    Query(key): Query<KeyParm>,
) -> impl IntoResponse {
    let (tx, rx) = channel();
    let action = HookManagerAction::List(tx, key.key);
    send_data_request!(action, injected.hook_sender);

    match rx.recv() {
        Ok(response) => match response {
            HookManagerResponse::HookList(hooks) => {
                let mut collection: Vec<Hook> = Vec::new();

                for (prefix, links) in hooks {
                    collection.push(Hook {
                        prefix: prefix,
                        links: links,
                    });
                }

                return_ok_with_value!(collection);
            }
            HookManagerResponse::Error(e) => return_client_error!(e),
            _ => return_server_error!(
                "this should happen request should have return HookList or Error"
            ),
        },
        Err(e) => return_server_error!(e),
    }
}

/// SUSPEND LOG
async fn suspend_log(State(injected): State<InjectedData>) -> impl IntoResponse {
    let (tx, rx) = channel();
    let action = LoggerAction::Suspend(tx);

    let sender = match &injected.logger_sender {
        Some(sender) => sender,
        None => return_server_error!("suspend was request but logger is off"),
    };
    send_data_request!(action, sender);

    match rx.recv() {
        Ok(response) => match response {
            LoggerResponse::Ok => return_ok!(),
            LoggerResponse::Err(e) => return_client_error!(e),
        },
        Err(e) => return_server_error!(e),
    }
}

/// RESUME LOG
async fn resume_log(State(injected): State<InjectedData>) -> impl IntoResponse {
    let (tx, rx) = channel();
    let action = LoggerAction::Resume(tx);

    let sender = match &injected.logger_sender {
        Some(sender) => sender,
        None => return_server_error!("suspend was request but logger is off"),
    };
    send_data_request!(action, sender);

    match rx.recv() {
        Ok(response) => match response {
            LoggerResponse::Ok => return_ok!(),
            LoggerResponse::Err(e) => return_client_error!(e),
        },
        Err(e) => return_server_error!(e),
    }
}

/// Start the REST server
pub async fn run_async(
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
    address: String,
    hook_sender: Arc<Mutex<Sender<HookManagerAction>>>,
    logger_sender: Option<Arc<Mutex<Sender<LoggerAction>>>>,
) {
    tracing::info!("REST interface on {} is starting...", address);

    let app = Router::new()
        .route("/db", post(set_key))
        .route("/db", get(get_key))
        .route("/db", delete(delete_key))
        .route("/db_list", get(list_keys))
        .route("/hook", post(set_hook))
        .route("/hook", get(get_hook))
        .route("/hook", delete(delete_hook))
        .route("/hook_list", get(list_hooks))
        .route("/logger/suspend", post(suspend_log))
        .route("/logger/resume", post(resume_log))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            tracing::error!("Unhandled internal error: {}", error),
                        ))
                    }
                }))
                .timeout(std::time::Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .with_state(InjectedData {
            data_sender,
            hook_sender,
            logger_sender,
        });

    let address: SocketAddr = address.parse().expect("Unable to parse REST api address");

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
