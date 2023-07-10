macro_rules! send_data_request {
    ($payload:expr, $data_mutex:expr) => {{
        let sender = $data_mutex.lock().await;
        if let Err(e) = sender.send($payload).await {
            for line in e.to_string().lines() {
                tracing::error!("{}", line);
            }
            return ">Err\n".as_bytes().to_vec();
        }
    }};
}
pub(in crate::server::interfaces::classic) use send_data_request;

macro_rules! return_server_error {
    ($error:expr) => {{
        for line in $error.to_string().lines() {
            tracing::error!("{}", line);
        }
        return ">Err".as_bytes().to_vec();
    }};
}
pub(in crate::server::interfaces::classic) use return_server_error;

macro_rules! return_client_error {
    ($error:expr) => {{
        return format!(">Err\n{}\n", $error).as_bytes().to_vec();
    }};
}
pub(in crate::server::interfaces::classic) use return_client_error;

macro_rules! return_ok {
    () => {{
        return ">Ok\n".as_bytes().to_vec();
    }};
}
pub(in crate::server::interfaces::classic) use return_ok;

macro_rules! return_ok_with_value {
    ($value:expr) => {{
        return format!(">Ok\n{}\n", $value).as_bytes().to_vec();
    }};
}
pub(in crate::server::interfaces::classic) use return_ok_with_value;
