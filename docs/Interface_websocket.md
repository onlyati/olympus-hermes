# Websocket interface

All action can be executed via websocket interface than on the other without any restriction. Interface expect JSON as input and it also send JSON back as response.

This is the struct that parse the input for websocket
```rust
/// Struct to parse request that are coming via websocket interface
#[derive(Serialize, Deserialize)]
pub(super) struct WsRequest {
    /// Command that tells what has to be done
    pub command: CommandMethod,

    /// Key for GET, SET, REM-KEY, REM-PATH, LIST-KEYS commands
    pub key: Option<String>,

    /// Value belongs to key
    pub value: Option<String>,

    /// Prefix for GET-HOOK, SET-HOOK, REM-HOOK, LIST-HOOKS commands
    pub prefix: Option<String>,
    /// Link belongs to prefix
    pub link: Option<String>,

    /// Stored procedure script
    pub exec: Option<String>,
    /// Parameter for stored procedure
    pub parm: Option<String>,
    /// Save the result of procedure or just a trigger
    pub save: Option<bool>,
}
```

Struct for response:
```rust
/// Struct to send response back for websocket calls
#[derive(Serialize, Deserialize)]
pub(super) struct WsResponse {
    /// Store that it is successful (Ok) or failed (Err)
    pub status: WsResponseStatus,

    /// If it is successful then return with the output.
    /// If it is failed then error message
    pub message: String,
}
```
