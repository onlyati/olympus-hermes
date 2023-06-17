use serde::{Serialize, Deserialize};

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

impl WsRequest {
    /// Parse `WsRequest` from text which must be a JSON
    /// 
    /// # Paramaters
    /// - `text`: This must be JSON and should be able to serialize as `WsRequest` structure
    pub fn from(text: &str) -> Result<Self, String> {
        match serde_json::from_str(text) {
            Ok(value) => return Ok(value),
            Err(e) => return Err(e.to_string()),
        }
    }
}

/// Enum for `WsRequest` structure that indicates the action type
#[derive(Serialize, Deserialize)]
pub(super) enum CommandMethod {
    GetKey,
    SetKey,
    RemKey,
    RemPath,
    ListKeys,
    Trigger,
    GetHook,
    SetHook,
    RemHook,
    ListHooks,
    SuspendLog,
    ResumeLog,
    Exec,
    Push,
    Pop,
}

/// Struct to send response back for websocket calls
#[derive(Serialize, Deserialize)]
pub(super) struct WsResponse {
    /// Store that it is successful (Ok) or failed (Err)
    pub status: WsResponseStatus,

    /// If it is successful then return with the output.
    /// If it is failed then error message
    pub message: String,
}

impl WsResponse {
    /// Create a new successful response
    pub fn new_ok<T: std::fmt::Display>(message: T) -> Self {
        return WsResponse { status: WsResponseStatus::Ok, message: message.to_string() }
    }

    /// Create a new failed response
    pub fn new_err<T: std::fmt::Display>(message: T) -> Self {
        return WsResponse { status: WsResponseStatus::Err, message: message.to_string() }
    }
}

/// Enum to indicate the status of websocket request
#[derive(Serialize, Deserialize)]
pub(super) enum WsResponseStatus {
    /// Successfully done
    Ok,

    /// Something went wrong
    Err,
}
