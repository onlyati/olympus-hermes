macro_rules! send_data_request {
    ($payload:expr, $data_mutex:expr) => {
        let req_status = {
            let sender = $data_mutex.lock().unwrap();
            sender.send($payload)
        };

        if let Err(e) = req_status {
            eprintln!("Error: {}", e);
            return ">Err\n".as_bytes().to_vec();
        }
    };
}
pub (in crate::interfaces::classic) use send_data_request;

macro_rules! return_server_error {
    ($error:expr) => {{
        eprintln!("Error: {}", $error);
        return ">Err".as_bytes().to_vec();
    }};
}
pub (in crate::interfaces::classic) use return_server_error;

macro_rules! return_client_error {
    ($error:expr) => {{
        return format!(">Err\n{}\n", $error).as_bytes().to_vec();
    }};
}
pub (in crate::interfaces::classic) use return_client_error;

macro_rules! return_ok {
    () => {{
        return ">Ok\n".as_bytes().to_vec();
    }};
}
pub (in crate::interfaces::classic) use return_ok;

macro_rules! return_ok_with_value {
    ($value:expr) => {{
        return format!(">Ok\n{}\n", $value).as_bytes().to_vec();
    }};
}
pub (in crate::interfaces::classic) use return_ok_with_value;
