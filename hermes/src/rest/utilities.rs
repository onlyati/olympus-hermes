use axum::error_handling::HandleErrorLayer;
use axum::extract::Path;
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
use std::sync::{mpsc::Sender, Arc, Mutex};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use onlyati_datastore::{enums::DatabaseAction, enums::ValueType, utilities};

use crate::rest::macros::return_ok;

use super::macros::{
    return_client_error, return_ok_with_value, return_server_error, send_data_request,
};

#[derive(Clone)]
pub struct InjectedData {
    data_sender: Arc<Mutex<Sender<DatabaseAction>>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Pair {
    key: String,
    value: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KeyParm {
    key: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeleteParm {
    key: String,
    kind: Option<String>,
}

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

async fn list_keys(
    State(injected): State<InjectedData>,
    Query(parms): Query<KeyParm>,
) -> impl IntoResponse {
    let (tx, rx) = utilities::get_channel_for_list();
    let list_action =
        DatabaseAction::ListKeys(tx, parms.key, onlyati_datastore::enums::ListType::All);

    send_data_request!(list_action, injected.data_sender);

    match rx.recv() {
        Ok(response) => match response {
            Ok(list) => return_ok_with_value!(list.iter().map(|x| x.get_key().clone()).collect::<Vec<String>>()),
            Err(e) => return_client_error!(e.to_string()),
        },
        Err(e) => return_server_error!(e),
    }
}

pub async fn run_async(data_sender: Arc<Mutex<Sender<DatabaseAction>>>, address: String) {
    println!("REST interface on {} is starting...", address);

    let app = Router::new()
        .route("/db", post(set_key))
        .route("/db", get(get_key))
        .route("/db", delete(delete_key))
        .route("/db_list", get(list_keys))
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
            data_sender: data_sender,
        });

    let address: SocketAddr = address.parse().expect("Unable to parse REST api address");

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
