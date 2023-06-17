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

macro_rules! verify_one_item {
    ($req:expr, $error_msg:expr) => {
        match $req {
            Some(value1) => value1,
            _ => return WsResponse::new_err($error_msg),
        }
    };
}
pub(in crate::server::interfaces::websocket) use verify_one_item;

macro_rules! verify_two_items {
    ($req1:expr, $req2:expr, $error_msg:expr) => {
        match ($req1, $req2) {
            (Some(value1), Some(value2)) => (value1, value2),
            _ => return WsResponse::new_err($error_msg),
        }
    };
}
pub(in crate::server::interfaces::websocket) use verify_two_items;
