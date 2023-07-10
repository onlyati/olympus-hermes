macro_rules! send_data_request {
    ($payload:expr, $data_mutex:expr) => {{
        let sender = $data_mutex.lock().await;
        if let Err(e) = sender.send($payload).await {
            for line in e.to_string().lines() {
                tracing::error!("{}", line);
            }
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }};
}
pub(in crate::server::interfaces::rest) use send_data_request;

macro_rules! return_server_error {
    ($error:expr) => {{
        for line in $error.to_string().lines() {
            tracing::error!("{}", line);
        }
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }};
}
pub(in crate::server::interfaces::rest) use return_server_error;

macro_rules! return_client_error {
    ($error:expr) => {{
        return (StatusCode::BAD_REQUEST, Json($error)).into_response();
    }};
}
pub(in crate::server::interfaces::rest) use return_client_error;

macro_rules! return_ok_with_value {
    ($value:expr) => {{
        return (StatusCode::OK, Json($value)).into_response();
    }};
}
pub(in crate::server::interfaces::rest) use return_ok_with_value;

macro_rules! return_ok {
    () => {{
        return StatusCode::OK.into_response();
    }};
}
pub(in crate::server::interfaces::rest) use return_ok;
