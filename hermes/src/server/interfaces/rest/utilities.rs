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
use std::sync::RwLock;
use std::sync::{mpsc::Sender, Arc, Mutex};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

// Internal depencies
use onlyati_datastore::datastore::{enums::pair::ValueType, enums::DatabaseAction, utilities};

use crate::server::utilities::config_parse::Config;

// Import macroes
use super::macros::{
    return_client_error, return_ok, return_ok_with_value, return_server_error, send_data_request,
};

/// Struct that is injected into every endpoint
#[derive(Clone)]
pub struct InjectedData {
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
    config: Arc<RwLock<Config>>,
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

/// Struct is used to query the EXEC endpoints
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExecArg {
    key: String,
    value: String,
    parms: Option<String>,
}

/// Struct is used to query the EXEC endpoints
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExecParm {
    exec: String,
    save: bool,
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

/// TRIGGER endpoint
async fn trigger(
    State(injected): State<InjectedData>,
    Json(pair): Json<Pair>,
) -> impl IntoResponse {
    let (tx, rx) = utilities::get_channel_for_set();
    let trigger_action = DatabaseAction::Trigger(tx, pair.key.clone(), pair.value.clone());

    send_data_request!(trigger_action, injected.data_sender);

    match rx.recv() {
        Ok(response) => match response {
            Ok(_) => return_ok!(),
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
    let action = DatabaseAction::HookSet(tx, pair.key, pair.value);
    send_data_request!(action, injected.data_sender);

    match rx.recv() {
        Ok(response) => match response {
            Ok(_) => return_ok!(),
            Err(e) => return_client_error!(e.to_string()),
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
    let action = DatabaseAction::HookGet(tx, key.key);
    send_data_request!(action, injected.data_sender);

    match rx.recv() {
        Ok(response) => match response {
            Ok((prefix, links)) => {
                return_ok_with_value!(Hook {
                    prefix: prefix,
                    links: links
                });
            }
            Err(e) => return_client_error!(e.to_string()),
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
    let action = DatabaseAction::HookRemove(tx, pair.key, pair.value);
    send_data_request!(action, injected.data_sender);

    match rx.recv() {
        Ok(response) => match response {
            Ok(_) => return_ok!(),
            Err(e) => return_client_error!(e.to_string()),
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
    let action = DatabaseAction::HookList(tx, key.key);
    send_data_request!(action, injected.data_sender);

    match rx.recv() {
        Ok(response) => match response {
            Ok(hooks) => {
                let mut collection: Vec<Hook> = Vec::new();

                for (prefix, links) in hooks {
                    collection.push(Hook {
                        prefix: prefix,
                        links: links,
                    });
                }

                return_ok_with_value!(collection);
            }
            Err(e) => return_client_error!(e.to_string()),
        },
        Err(e) => return_server_error!(e),
    }
}

/// SUSPEND LOG
async fn suspend_log(State(injected): State<InjectedData>) -> impl IntoResponse {
    let (tx, rx) = channel();
    let action = DatabaseAction::SuspendLog(tx);

    send_data_request!(action, injected.data_sender);

    match rx.recv() {
        Ok(response) => match response {
            Ok(_) => return_ok!(),
            Err(e) => return_client_error!(e.to_string()),
        },
        Err(e) => return_server_error!(e),
    }
}

/// RESUME LOG
async fn resume_log(State(injected): State<InjectedData>) -> impl IntoResponse {
    let (tx, rx) = channel();
    let action = DatabaseAction::ResumeLog(tx);

    send_data_request!(action, injected.data_sender);

    match rx.recv() {
        Ok(response) => match response {
            Ok(_) => return_ok!(),
            Err(e) => return_client_error!(e.to_string()),
        },
        Err(e) => return_server_error!(e),
    }
}

/// EXEC_SET
async fn exec_script(
    State(injected): State<InjectedData>,
    Query(exec): Query<ExecParm>,
    Json(arg): Json<ExecArg>,
) -> impl IntoResponse {
    // Get the old value of exists
    let (tx, rx) = utilities::get_channel_for_get();
    let get_action = DatabaseAction::Get(tx, arg.key.clone());

    send_data_request!(get_action, injected.data_sender);

    let old_pair = match rx.recv() {
        Ok(response) => match response {
            Ok(value) => match value {
                ValueType::RecordPointer(data) => Some((arg.key.clone(), data.clone())),
                _ => return_server_error!("Pointer must be Record but it was Table"),
            },
            Err(_) => None,
        },
        Err(e) => return_server_error!(e),
    };

    // Get config
    let config = match injected.config.read() {
        Ok(cfg) => match &cfg.scripts {
            Some(scr) => match scr.execs.contains(&exec.exec) {
                true => scr.clone(),
                false => return_client_error!("requested script is not defined"),
            },
            None => return_client_error!("requested script is not defined"),
        },
        Err(_) => {
            tracing::error!("RwLock for config has poisned");
            panic!("RwLock for config has poisned");
        }
    };

    let new_pair = (arg.key.clone(), arg.value.clone());

    // Call lua utility
    let modified_pair =
        match crate::server::utilities::lua::run(config, old_pair, new_pair, exec.exec, arg.parms).await {
            Ok(modified_pair) => modified_pair,
            Err(e) => return_server_error!(format!("error during script exection: {}", e)),
        };

    // Make a SET action for the modified pair
    if exec.save {
        if modified_pair.1.is_empty() {
            let (tx, rx) = utilities::get_channel_for_delete();

            let action = DatabaseAction::DeleteKey(tx, modified_pair.0);
            send_data_request!(action, injected.data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(_) => return_ok!(),
                    Err(e) => return_client_error!(e.to_string()),
                },
                Err(e) => return_server_error!(e),
            }
        } else {
            let (tx, rx) = channel();
            let action = DatabaseAction::Set(tx, modified_pair.0, modified_pair.1);
            send_data_request!(action, injected.data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(_) => return_ok!(),
                    Err(e) => return_client_error!(e.to_string()),
                },
                Err(e) => return_server_error!(e),
            }
        }
    }
    // Or a TRIGGER if this was requested
    else {
        if !modified_pair.1.is_empty() {
            let (tx, rx) = channel();
            let action = DatabaseAction::Trigger(tx, modified_pair.0, modified_pair.1);
            send_data_request!(action, injected.data_sender);

            match rx.recv() {
                Ok(response) => match response {
                    Ok(_) => return_ok!(),
                    Err(e) => return_client_error!(e.to_string()),
                },
                Err(e) => return_server_error!(e),
            }
        } else {
            return_client_error!("After script was run, the new value is empty");
        }
    }
}

/// Start the REST server
pub async fn run_async(
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
    address: String,
    config: Arc<RwLock<Config>>,
) {
    tracing::info!("REST interface on {} is starting...", address);

    let app = Router::new()
        .route("/db", post(set_key))
        .route("/db", get(get_key))
        .route("/db", delete(delete_key))
        .route("/db_list", get(list_keys))
        .route("/trigger", post(trigger))
        .route("/hook", post(set_hook))
        .route("/hook", get(get_hook))
        .route("/hook", delete(delete_hook))
        .route("/hook_list", get(list_hooks))
        .route("/logger/suspend", post(suspend_log))
        .route("/logger/resume", post(resume_log))
        .route("/exec", post(exec_script))
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
            config,
        });

    let address: SocketAddr = address.parse().expect("Unable to parse REST api address");

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
