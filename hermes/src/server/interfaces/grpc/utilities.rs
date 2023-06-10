// External depencies
use axum::error_handling::HandleErrorLayer;
use reqwest::StatusCode;
use std::sync::RwLock;
use std::sync::{mpsc::channel, mpsc::Sender, Arc, Mutex};
use tonic::{transport::Server, Request, Response, Status};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;

// Internal depencies
use hermes::hermes_server::{Hermes, HermesServer};
use hermes::{Empty, ExecArg, Hook, HookCollection, Key, KeyList, LinkCollection, Pair};
use onlyati_datastore::datastore::{enums::pair::ValueType, enums::DatabaseAction, utilities};

use crate::server::utilities::config_parse::Config;

// Import macros
use super::macros::{
    check_self_sender, return_client_error, return_ok_with_value, return_server_error,
    send_data_request,
};

// Generate structs for gRPC
mod hermes {
    tonic::include_proto!("hermes");
}

/// Struct that is injected into every gRPC endpoint
#[derive(Debug, Default)]
struct HermesGrpc {
    data_sender: Option<Arc<Mutex<Sender<DatabaseAction>>>>,
    config: Arc<RwLock<Config>>,
}

/// gRPC endpoints
#[tonic::async_trait]
impl Hermes for HermesGrpc {
    /// Endpoint for SET request
    async fn set(&self, request: Request<Pair>) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();
        let data_sender = check_self_sender!(&self.data_sender);

        let (tx, rx) = utilities::get_channel_for_set();
        let set_action = DatabaseAction::Set(tx, request.key, request.value);
        send_data_request!(set_action, data_sender);

        match rx.recv() {
            Ok(response) => match response {
                Ok(_) => return_ok_with_value!(Empty::default()),
                Err(e) => return_client_error!(e.to_string()),
            },
            Err(e) => return_server_error!(e),
        }
    }

    /// Endpoint for GET request
    async fn get(&self, request: Request<Key>) -> Result<Response<Pair>, Status> {
        let request = request.into_inner();
        let data_sender = check_self_sender!(&self.data_sender);

        let (tx, rx) = utilities::get_channel_for_get();
        let get_action = DatabaseAction::Get(tx, request.key.clone());
        send_data_request!(get_action, data_sender);

        match rx.recv() {
            Ok(response) => match response {
                Ok(value) => match value {
                    ValueType::RecordPointer(data) => return_ok_with_value!(Pair {
                        key: request.key,
                        value: data
                    }),
                    _ => return_server_error!("Pointer must be Record but it was Table"),
                },
                Err(e) => return_client_error!(e.to_string()),
            },
            Err(e) => return_server_error!(e),
        }
    }

    /// Endpoint for REMKEY request
    async fn delete_key(&self, request: Request<Key>) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();
        let data_sender = check_self_sender!(&self.data_sender);

        let (tx, rx) = utilities::get_channel_for_delete();
        let rem_action = DatabaseAction::DeleteKey(tx, request.key);
        send_data_request!(rem_action, data_sender);

        match rx.recv() {
            Ok(response) => match response {
                Ok(_) => return_ok_with_value!(Empty::default()),
                Err(e) => return_client_error!(e.to_string()),
            },
            Err(e) => return_server_error!(e),
        }
    }

    /// Endpoint for REMPATH request
    async fn delete_path(&self, request: Request<Key>) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();
        let data_sender = check_self_sender!(&self.data_sender);

        let (tx, rx) = utilities::get_channel_for_delete();
        let rem_action = DatabaseAction::DeleteTable(tx, request.key);
        send_data_request!(rem_action, data_sender);

        match rx.recv() {
            Ok(response) => match response {
                Ok(_) => return_ok_with_value!(Empty::default()),
                Err(e) => return_client_error!(e.to_string()),
            },
            Err(e) => return_server_error!(e),
        }
    }

    /// Endpoint for LIST request
    async fn list_keys(&self, request: Request<Key>) -> Result<Response<KeyList>, Status> {
        let request = request.into_inner();
        let data_sender = check_self_sender!(&self.data_sender);

        let (tx, rx) = utilities::get_channel_for_list();
        let list_action = DatabaseAction::ListKeys(
            tx,
            request.key,
            onlyati_datastore::datastore::enums::ListType::All,
        );
        send_data_request!(list_action, data_sender);

        match rx.recv() {
            Ok(response) => match response {
                Ok(value) => return_ok_with_value!(KeyList {
                    keys: value.iter().map(|x| x.get_key().to_string()).collect(),
                }),
                Err(e) => return_client_error!(e.to_string()),
            },
            Err(e) => return_server_error!(e),
        }
    }

    /// Endpoint for TRIGGER request
    async fn trigger(&self, request: Request<Pair>) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();
        let data_sender = check_self_sender!(&self.data_sender);

        let (tx, rx) = utilities::get_channel_for_set();
        let trigger_action = DatabaseAction::Trigger(tx, request.key, request.value);
        send_data_request!(trigger_action, data_sender);

        match rx.recv() {
            Ok(response) => match response {
                Ok(_) => return_ok_with_value!(Empty::default()),
                Err(e) => return_client_error!(e.to_string()),
            },
            Err(e) => return_server_error!(e),
        }
    }

    /// Create a new hook
    async fn set_hook(&self, request: Request<Pair>) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();
        let data_sender = check_self_sender!(&self.data_sender);

        let (tx, rx) = channel();
        let action = DatabaseAction::HookSet(tx, request.key, request.value);
        send_data_request!(action, data_sender);

        match rx.recv() {
            Ok(response) => match response {
                Ok(_) => return_ok_with_value!(Empty::default()),
                Err(e) => return_client_error!(e.to_string()),
            },
            Err(e) => return_server_error!(e),
        }
    }

    /// Remove existing hook
    async fn delete_hook(&self, request: Request<Pair>) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();
        let data_sender = check_self_sender!(&self.data_sender);

        let (tx, rx) = channel();
        let action = DatabaseAction::HookRemove(tx, request.key, request.value);
        send_data_request!(action, data_sender);

        match rx.recv() {
            Ok(response) => match response {
                Ok(_) => return_ok_with_value!(Empty::default()),
                Err(e) => return_client_error!(e.to_string()),
            },
            Err(e) => return_server_error!(e),
        }
    }

    /// Check that hook exist
    async fn get_hook(&self, request: Request<Key>) -> Result<Response<Hook>, Status> {
        let request = request.into_inner();
        let data_sender = check_self_sender!(&self.data_sender);

        let (tx, rx) = channel();
        let action = DatabaseAction::HookGet(tx, request.key);
        send_data_request!(action, data_sender);

        match rx.recv() {
            Ok(response) => match response {
                Ok((prefix, links)) => {
                    let collection = LinkCollection { links: links };
                    let hook = Hook {
                        prefix: prefix,
                        links: Some(collection),
                    };
                    return_ok_with_value!(hook);
                }
                Err(e) => return_client_error!(e.to_string()),
            },
            Err(e) => return_server_error!(e),
        }
    }

    /// List hooks under a prefix
    async fn list_hooks(&self, request: Request<Key>) -> Result<Response<HookCollection>, Status> {
        let request = request.into_inner();
        let data_sender = check_self_sender!(&self.data_sender);

        let (tx, rx) = channel();
        let action = DatabaseAction::HookList(tx, request.key);
        send_data_request!(action, data_sender);

        match rx.recv() {
            Ok(response) => match response {
                Ok(hooks) => {
                    let mut collection = HookCollection { hooks: Vec::new() };

                    for (prefix, links) in hooks {
                        let links = LinkCollection { links: links };
                        let hook = Hook {
                            prefix: prefix,
                            links: Some(links),
                        };
                        collection.hooks.push(hook);
                    }

                    return_ok_with_value!(collection);
                }
                Err(e) => return_client_error!(e.to_string()),
            },
            Err(e) => return_server_error!(e),
        }
    }

    /// Suspend logger
    async fn suspend_log(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let data_sender = check_self_sender!(&self.data_sender);

        let (tx, rx) = channel();
        let action = DatabaseAction::SuspendLog(tx);
        send_data_request!(action, data_sender);

        match rx.recv() {
            Ok(response) => match response {
                Ok(_) => return_ok_with_value!(Empty::default()),
                Err(e) => return_client_error!(e.to_string()),
            },
            Err(e) => return_server_error!(e),
        }
    }

    /// Resume logger
    async fn resume_log(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let data_sender = check_self_sender!(&self.data_sender);

        let (tx, rx) = channel();
        let action = DatabaseAction::ResumeLog(tx);
        send_data_request!(action, data_sender);

        match rx.recv() {
            Ok(response) => match response {
                Ok(_) => return_ok_with_value!(Empty::default()),
                Err(e) => return_client_error!(e.to_string()),
            },
            Err(e) => return_server_error!(e),
        }
    }

    /// Execute lua script and save the value
    async fn exec_script(&self, request: Request<ExecArg>) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();
        let data_sender = check_self_sender!(&self.data_sender);

        // Get the old value of exists
        let (tx, rx) = utilities::get_channel_for_get();
        let get_action = DatabaseAction::Get(tx, request.key.clone());

        send_data_request!(get_action, data_sender);

        let old_pair = match rx.recv() {
            Ok(response) => match response {
                Ok(value) => match value {
                    ValueType::RecordPointer(data) => Some((request.key.clone(), data.clone())),
                    _ => return_server_error!("Pointer must be Record but it was Table"),
                },
                Err(_) => None,
            },
            Err(e) => return_server_error!(e),
        };

        // Get config
        let config = match &self.config.read() {
            Ok(cfg) => match &cfg.scripts {
                Some(scr) => match scr.execs.contains(&request.exec) {
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

        let new_pair = (request.key.clone(), request.value.clone());

        let parms = match !request.parms.is_empty() {
            true => Some(request.parms.clone()),
            false => None,
        };

        // // Call lua utility
        let modified_pair =
            match crate::server::utilities::lua::run(config, old_pair, new_pair, request.exec, parms).await
            {
                Ok(modified_pair) => modified_pair,
                Err(e) => return_server_error!(format!("error during script exection: {}", e)),
            };

        // Make a SET action for the modified pair
        if request.save == true {
            if modified_pair.1.is_empty() {
                let (tx, rx) = utilities::get_channel_for_delete();

                let action = DatabaseAction::DeleteKey(tx, modified_pair.0);
                send_data_request!(action, data_sender);

                match rx.recv() {
                    Ok(response) => match response {
                        Ok(_) => return_ok_with_value!(Empty::default()),
                        Err(e) => return_client_error!(e.to_string()),
                    },
                    Err(e) => return_server_error!(e),
                }
            } else {
                let (tx, rx) = channel();
                let action = DatabaseAction::Set(tx, modified_pair.0, modified_pair.1);
                send_data_request!(action, data_sender);

                match rx.recv() {
                    Ok(response) => match response {
                        Ok(_) => return_ok_with_value!(Empty::default()),
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
                send_data_request!(action, data_sender);

                match rx.recv() {
                    Ok(response) => match response {
                        Ok(_) => return_ok_with_value!(Empty::default()),
                        Err(e) => return_client_error!(e.to_string()),
                    },
                    Err(e) => return_server_error!(e),
                }
            } else {
                return_client_error!("After script was run, the new value is empty");
            }
        }
    }

    /// Endpoint for PUSH request
    async fn push(&self, request: Request<Pair>) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();
        let data_sender = check_self_sender!(&self.data_sender);

        let (tx, rx) = channel();
        let set_action = DatabaseAction::Push(tx, request.key, request.value);
        send_data_request!(set_action, data_sender);

        match rx.recv() {
            Ok(response) => match response {
                Ok(_) => return_ok_with_value!(Empty::default()),
                Err(e) => return_client_error!(e.to_string()),
            },
            Err(e) => return_server_error!(e),
        }
    }

    /// Endpoint for POP request
    async fn pop(&self, request: Request<Key>) -> Result<Response<Pair>, Status> {
        let request = request.into_inner();
        let data_sender = check_self_sender!(&self.data_sender);

        let (tx, rx) = channel();
        let get_action = DatabaseAction::Pop(tx, request.key.clone());
        send_data_request!(get_action, data_sender);

        match rx.recv() {
            Ok(response) => match response {
                Ok(value) => match value {
                    ValueType::RecordPointer(data) => return_ok_with_value!(Pair {
                        key: request.key,
                        value: data
                    }),
                    _ => return_server_error!("Pointer must be Record but it was Table"),
                },
                Err(e) => return_client_error!(e.to_string()),
            },
            Err(e) => return_server_error!(e),
        }
    }
}

/// Start gRPC server
pub async fn run_async(
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
    address: String,
    config: Arc<RwLock<Config>>,
) {
    let mut hermes_grpc = HermesGrpc::default();
    hermes_grpc.data_sender = Some(data_sender);
    hermes_grpc.config = config;
    let hermes_service = HermesServer::new(hermes_grpc);

    tracing::info!("gRPC interface on {} is starting...", address);
    Server::builder()
        .accept_http1(true)
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
        .add_service(hermes_service)
        .serve(address.parse().unwrap())
        .await
        .unwrap();
}
