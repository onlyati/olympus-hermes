macro_rules! send_data_request {
    ($payload:expr, $data_mutex:expr) => {
        let req_status = {
            let sender = $data_mutex.lock().unwrap();
            sender.send($payload)
        };

        if let Err(e) = req_status {
            for line in e.to_string().lines() {
                tracing::error!("{}", line);
            }
            return WsResponse::new_err("internal server error");
        }
    };
}
pub(in crate::server::interfaces::websocket) use send_data_request;

macro_rules! send_data_back {
    ($socket: expr, $msg:expr) => {
        if let Err(e) = $socket.send($msg).await {
            tracing::error!("failed to send response");
            for line in e.to_string().lines() {
                tracing::error!("{}", line);
            }
        }
    };
}
pub(in crate::server::interfaces::websocket) use send_data_back;

macro_rules! verify_key {
    ($req:expr) => {
        match $req.key {
            Some(key) => key,
            _ => return WsResponse::new_err("'key' and 'value' must be specified for this command"),
        }
    };
}
pub(in crate::server::interfaces::websocket) use verify_key;

macro_rules! verify_key_value {
    ($req:expr) => {
        match ($req.key, $req.value) {
            (Some(key), Some(value)) => (key, value),
            _ => return WsResponse::new_err("'key' and 'value' must be specified for this command"),
        }
    };
}
pub(in crate::server::interfaces::websocket) use verify_key_value;