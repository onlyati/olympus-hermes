// External depencies
use axum::error_handling::HandleErrorLayer;
use reqwest::StatusCode;
use std::sync::{mpsc::Sender, Arc, Mutex};
use tonic::{transport::Server, Request, Response, Status};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;

// Internal depencies
use hermes::hermes_server::{Hermes, HermesServer};
use hermes::{Empty, Key, KeyList, Pair};
use onlyati_datastore::datastore::{enums::pair::ValueType, enums::DatabaseAction, utilities};

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
}

/// Start gRPC server
pub async fn run_async(data_sender: Arc<Mutex<Sender<DatabaseAction>>>, address: String) {
    let mut hermes_grpc = HermesGrpc::default();
    hermes_grpc.data_sender = Some(data_sender);
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
