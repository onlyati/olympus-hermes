macro_rules! send_data_request {
    ($payload:expr, $data_mutex:expr) => {
        let req_status = {
            let sender = $data_mutex.lock().unwrap();
            sender.send($payload)
        };

        if let Err(e) = req_status {
            eprintln!("Error: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
}
pub(super) use send_data_request;

macro_rules! return_server_error {
    ($error:expr) => {{
        eprintln!("Error: {}", $error);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }};
}
pub(super) use return_server_error;

macro_rules! return_client_error {
    ($error:expr) => {{
        return (StatusCode::BAD_REQUEST, Json($error)).into_response();
    }};
}
pub(super) use return_client_error;

macro_rules! return_ok_with_value {
    ($value:expr) => {{
        return (StatusCode::OK, Json($value)).into_response();
    }};
}
pub(super) use return_ok_with_value;

macro_rules! return_ok {
    () => {{
        return StatusCode::OK.into_response();
    }};
}
pub(super) use return_ok;
