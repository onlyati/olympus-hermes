macro_rules! send_data_back {
    ($socket: expr, $msg:expr) => {
        if let Err(e) = $socket.send($msg).await {
            tracing::error!("failed to send response");
            for line in e.to_string().lines() {
                tracing::error!("{}", line);
            }
        };
    };
}
pub(in crate::server::interfaces::websocket) use send_data_back;
