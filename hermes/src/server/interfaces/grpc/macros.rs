macro_rules! send_data_request {
    ($payload:expr, $data_mutex:expr) => {
        let req_status = {
            let sender = $data_mutex.lock().unwrap();
            sender.send($payload)
        };

        if let Err(e) = req_status {
            tracing::error!("Error: {}", e);
            return Err(Status::internal("Internal server error"));
        }
    };
}
pub(in crate::interfaces::grpc) use send_data_request;

macro_rules! return_server_error {
    ($error:expr) => {{
        tracing::error!("Error: {}", $error);
        return Err(Status::internal("Internal server error"));
    }};
}
pub(in crate::interfaces::grpc) use return_server_error;

macro_rules! return_client_error {
    ($error:expr) => {{
        return Err(Status::invalid_argument($error));
    }};
}
pub(in crate::interfaces::grpc) use return_client_error;

macro_rules! return_ok_with_value {
    ($value:expr) => {{
        return Ok(Response::new($value));
    }};
}
pub(in crate::interfaces::grpc) use return_ok_with_value;

macro_rules! check_self_sender {
    ($self_sender:expr) => {
        match $self_sender {
            Some(sender) => sender,
            None => return_server_error!("Data sender for gRPC struct is node defined"),
        }
    };
}
pub(in crate::interfaces::grpc) use check_self_sender;
