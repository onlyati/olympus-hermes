// External depencies
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
    /// Sender to send data to database thread
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,

    /// Configuration of application
    config: Arc<RwLock<Config>>,
}

/// Struct is used to query the SET endpoint
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Pair {
    /// Key for record
    key: String,

    /// Value of key
    value: String,
}

/// Struct is used to query the GET and LIST endpoint
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KeyParm {
    /// Key for record
    key: String,
}

/// Struct is used to query the REMKEY and REMPATH endpoints
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeleteParm {
    /// Key for record
    key: String,

    /// Explain what must be deleted: `record` or `path`
    kind: Option<String>,
}

/// Struct that is used to return with hook value
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Hook {
    /// Prefix that belongs to a hook definition
    prefix: String,

    /// Link that belongs to a prefix
    links: Vec<String>,
}

/// Struct is used to query the EXEC endpoints
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExecArg {
    /// Key for record
    key: String,

    /// Value for key
    value: String,

    /// Parameters that is passed to lua script
    parms: Option<String>,
}

/// Struct is used to query the EXEC endpoints
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExecParm {
    /// Name of lua script that will be called
    exec: String,

    /// Result of the script should be saved like a set key or just use as trigger
    save: bool,
}

/// Endpoint to get value of a key
///
/// # Http parameters:
/// - Endpoint: `GET /db`
/// - Body: `none`
/// - Query: `?key=_string_`
///
/// # Other parameters
/// - `injected`: Axum state that share information among endpoints
///
/// # Return codes
/// - `OK`: Successfully done
/// - `BAD_REQUEST`: Something was specified badly in the request
/// - `INTERNAL_SERVER_ERROR`: Something issue happened on server
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

/// Endpoint to set value for a key, override value if already exist
///
/// # Http parameters:
/// - Endpoint: `POST /db`
/// - Body: `JSON { "key" : _string_, "value" : _string_ }`
/// - Query: none
///
/// # Other parameters
/// - `injected`: Axum state that share information among endpoints
///
/// # Return codes
/// - `OK`: Successfully done
/// - `BAD_REQUEST`: Something was specified badly in the request
/// - `INTERNAL_SERVER_ERROR`: Something issue happened on server
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

/// Endpoint to remove record or complete path
///
/// # Http parameters:
/// - Endpoint: `DELETE /db`
/// - Body: `none`
/// - Query: `?key=_string_&kind=_string_`
///   - Kind if optional and it can be `record` or `path`
///
/// # Other parameters
/// - `injected`: Axum state that share information among endpoints
///
/// # Return codes
/// - `OK`: Successfully done
/// - `BAD_REQUEST`: Something was specified badly in the request
/// - `INTERNAL_SERVER_ERROR`: Something issue happened on server
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

/// Endpoint to list keys that prefix match with the specified
///
/// # Http parameters:
/// - Endpoint: `GET /db_list`
/// - Body: `none`
/// - Query: `?key=_string_`
///
/// # Other parameters
/// - `injected`: Axum state that share information among endpoints
///
/// # Return codes
/// - `OK`: Successfully done
/// - `BAD_REQUEST`: Something was specified badly in the request
/// - `INTERNAL_SERVER_ERROR`: Something issue happened on server
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

/// Endpoint to issue a trigger. It does not save data but send pair to hook manager for check
///
/// # Http parameters:
/// - Endpoint: `POST /trigger`
/// - Body: `JSON { "key" : _string_, "value" : _string_ }`
/// - Query: `none`
///
/// # Other parameters
/// - `injected`: Axum state that share information among endpoints
///
/// # Return codes
/// - `OK`: Successfully done
/// - `BAD_REQUEST`: Something was specified badly in the request
/// - `INTERNAL_SERVER_ERROR`: Something issue happened on server
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

/// Endpoint to define a new hook
///
/// # Http parameters:
/// - Endpoint: `POST /hook`
/// - Body: `JSON { "key" : _string_, "value" : _string_ }`
/// - Query: `none`
///
/// # Other parameters
/// - `injected`: Axum state that share information among endpoints
///
/// # Return codes
/// - `OK`: Successfully done
/// - `BAD_REQUEST`: Something was specified badly in the request
/// - `INTERNAL_SERVER_ERROR`: Something issue happened on server
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

/// Endpoint to get a hook information
///
/// # Http parameters:
/// - Endpoint: `GET /hook`
/// - Body: `none`
/// - Query: `?key=_string_`
///
/// # Other parameters
/// - `injected`: Axum state that share information among endpoints
///
/// # Return codes
/// - `OK`: Successfully done
/// - `BAD_REQUEST`: Something was specified badly in the request
/// - `INTERNAL_SERVER_ERROR`: Something issue happened on server
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

/// Endpoint to remove a hook
///
/// # Http parameters:
/// - Endpoint: `DELETE /hook`
/// - Body: `none`
/// - Query: `?key=_string_`
///
/// # Other parameters
/// - `injected`: Axum state that share information among endpoints
///
/// # Return codes
/// - `OK`: Successfully done
/// - `BAD_REQUEST`: Something was specified badly in the request
/// - `INTERNAL_SERVER_ERROR`: Something issue happened on server
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

/// Endpoint to list hooks under a specified prefix
///
/// # Http parameters:
/// - Endpoint: `GET /hook_list`
/// - Body: `none`
/// - Query: `?key=_string_`
///
/// # Other parameters
/// - `injected`: Axum state that share information among endpoints
///
/// # Return codes
/// - `OK`: Successfully done
/// - `BAD_REQUEST`: Something was specified badly in the request
/// - `INTERNAL_SERVER_ERROR`: Something issue happened on server
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

/// Endpoint to suspend the database logging
///
/// # Http parameters:
/// - Endpoint: `POST /logger/suspend`
/// - Body: `none`
/// - Query: `none`
///
/// # Other parameters
/// - `injected`: Axum state that share information among endpoints
///
/// # Return codes
/// - `OK`: Successfully done
/// - `BAD_REQUEST`: Something was specified badly in the request
/// - `INTERNAL_SERVER_ERROR`: Something issue happened on server
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

/// Endpoint to resume the database logging
///
/// # Http parameters:
/// - Endpoint: `POST /logger/resume`
/// - Body: `none`
/// - Query: `none`
///
/// # Other parameters
/// - `injected`: Axum state that share information among endpoints
///
/// # Return codes
/// - `OK`: Successfully done
/// - `BAD_REQUEST`: Something was specified badly in the request
/// - `INTERNAL_SERVER_ERROR`: Something issue happened on server
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

/// Endpoint to suspend the database logging
///
/// # Http parameters:
/// - Endpoint: `POST /exec`
/// - Body: `JSON { "key" : _string, "value" : _string_, "parms" : _string_ }`
///   - `parms` is optional
/// - Query: `?exec=_string_&save=_bool_`
///
/// # Other parameters
/// - `injected`: Axum state that share information among endpoints
///
/// # Return codes
/// - `OK`: Successfully done
/// - `BAD_REQUEST`: Something was specified badly in the request
/// - `INTERNAL_SERVER_ERROR`: Something issue happened on server
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
        match crate::server::utilities::lua::run(config, old_pair, new_pair, exec.exec, arg.parms)
            .await
        {
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

/// Endpoint to check that interface work
///
/// # Http parameters:
/// - Endpoint: `GET /hc`
/// - Body: `none`
/// - Query: `none`
///
/// # Return codes
/// - `OK`: Successfully done
/// - `BAD_REQUEST`: Something was specified badly in the request
/// - `INTERNAL_SERVER_ERROR`: Something issue happened on server
pub async fn health_check() -> impl IntoResponse {
    return_ok!();
}

/// Endpoint to push item into a queue
///
/// # Http parameters:
/// - Endpoint: `POST /queue`
/// - Body: `JSON { "key" : _string, "value" : _string_, "parms" : _string_ }`
/// - Query: `none`
///
/// # Other parameters
/// - `injected`: Axum state that share information among endpoints
///
/// # Return codes
/// - `OK`: Successfully done
/// - `BAD_REQUEST`: Something was specified badly in the request
/// - `INTERNAL_SERVER_ERROR`: Something issue happened on server
async fn push(State(injected): State<InjectedData>, Json(pair): Json<Pair>) -> impl IntoResponse {
    let (tx, rx) = channel();
    let set_action = DatabaseAction::Push(tx, pair.key.clone(), pair.value.clone());

    send_data_request!(set_action, injected.data_sender);

    match rx.recv() {
        Ok(response) => match response {
            Ok(_) => return_ok!(),
            Err(e) => return_client_error!(e.to_string()),
        },
        Err(e) => return_server_error!(e),
    }
}

/// Endpoint to get item from a queue
///
/// # Http parameters:
/// - Endpoint: `GET /queue`
/// - Body: `none`
/// - Query: `?key=_string_`
///
/// # Other parameters
/// - `injected`: Axum state that share information among endpoints
///
/// # Return codes
/// - `OK`: Successfully done
/// - `BAD_REQUEST`: Something was specified badly in the request
/// - `INTERNAL_SERVER_ERROR`: Something issue happened on server
async fn pop(
    State(injected): State<InjectedData>,
    Query(parms): Query<KeyParm>,
) -> impl IntoResponse {
    let (tx, rx) = channel();
    let get_action = DatabaseAction::Pop(tx, parms.key);

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

/// Start the REST server
/// 
/// # Parameters
/// - `data_sender`: Sender that send data to database thread
/// - `address`: Host address where interface bind and listen
/// - `config`: Configuration of the application
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
        .route("/hc", get(health_check))
        .route("/queue", post(push))
        .route("/queue", get(pop))
        .layer(tower_http::timeout::TimeoutLayer::new(
            std::time::Duration::from_secs(10),
        ))
        .layer(tower_http::trace::TraceLayer::new_for_http())
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
        .serve(app.into_make_service())
        .await
    {
        tracing::error!("failed to start REST server");
        for line in e.to_string().lines() {
            tracing::error!("{}", line);
        }
    }
}
