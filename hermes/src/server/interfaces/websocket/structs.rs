use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub(super) struct WsRequest {
    pub command: CommandMethod,   // Command that tells what has to be done

    pub key: Option<String>,      // Key for GET, SET, REM-KEY, REM-PATH, LIST-KEYS commands
    pub value: Option<String>,    // Value belongs to key

    pub prefix: Option<String>,   // Prefix for GET-HOOK, SET-HOOK, REM-HOOK, LIST-HOOKS commands
    pub link: Option<String>,     // Link belongs to prefix

    pub exec: Option<String>,     // Stored procedure script
    pub parm: Option<String>,     // Parameter for stored procedure
    pub save: Option<bool>        // Save the result of procedure or just a trigger
}

impl WsRequest {
    pub fn from(text: &str) -> Result<Self, String> {
        match serde_json::from_str(text) {
            Ok(value) => return Ok(value),
            Err(e) => return Err(e.to_string()),
        }
    }
}

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
