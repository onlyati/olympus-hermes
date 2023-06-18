use futures_util::SinkExt;
use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::arg::Action;
use crate::common::websocket::CommandMethod;

pub async fn connecto_to_server(
    hostname: String,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, String> {
    let hostname = hostname + "/ws";

    let url = match url::Url::parse(&hostname[..]) {
        Ok(url) => url,
        Err(e) => return Err(e.to_string()),
    };

    let (stream, _) = match connect_async(url).await {
        Ok(stream) => stream,
        Err(e) => return Err(e.to_string()),
    };

    Ok(stream)
}

pub async fn perform_action(
    socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    action: Action,
) -> Result<String, String> {
    let mut request = crate::common::websocket::WsRequest::default();

    match &action {
        // GET action
        Action::Get { key } => {
            request.command = CommandMethod::GetKey;
            request.key = Some(key.clone());
        }
        // SET action
        Action::Set { key, value } => {
            request.command = CommandMethod::SetKey;
            request.key = Some(key.clone());
            request.value = Some(value.clone());
        }
        // REMKEY action
        Action::RemKey { key } => {
            request.command = CommandMethod::RemKey;
            request.key = Some(key.clone());
        }
        // REMPATH action
        Action::RemPath { key } => {
            request.command = CommandMethod::RemPath;
            request.key = Some(key.clone());
        }
        // LIST action
        Action::ListKeys { key } => {
            request.command = CommandMethod::ListKeys;
            request.key = Some(key.clone());
        }
        // TRIGGER action
        Action::Trigger { key, value } => {
            request.command = CommandMethod::Trigger;
            request.key = Some(key.clone());
            request.value = Some(value.clone());
        }
        // GETHOOK action
        Action::GetHook { prefix } => {
            request.command = CommandMethod::GetHook;
            request.prefix = Some(prefix.clone());
        }
        // SETHOOK action
        Action::SetHook { prefix, link } => {
            request.command = CommandMethod::SetHook;
            request.prefix = Some(prefix.clone());
            request.link = Some(link.clone());
        }
        // REMHOOK action
        Action::RemHook { prefix, link } => {
            request.command = CommandMethod::RemHook;
            request.prefix = Some(prefix.clone());
            request.link = Some(link.clone());
        }
        // LISTHOOK action
        Action::ListHooks { prefix } => {
            request.command = CommandMethod::ListHooks;
            request.prefix = Some(prefix.clone());
        }
        // SUSPEND LOG action
        Action::SuspendLog => {
            request.command = CommandMethod::SuspendLog;
        }
        // RESUME LOG action
        Action::ResumeLog => {
            request.command = CommandMethod::ResumeLog;
        }
        // Execute script
        Action::Exec {
            key,
            value,
            script,
            parms,
            save,
        } => {
            request.command = CommandMethod::Exec;
            request.key = Some(key.clone());
            request.value = Some(value.clone());
            request.exec = Some(script.clone());
            request.parm = parms.clone();
            request.save = Some(*save);
        }
        // POP action
        Action::Pop { key } => {
            request.command = CommandMethod::Pop;
            request.key = Some(key.clone());
        }
        // PUSH action
        Action::Push { key, value } => {
            request.command = CommandMethod::Push;
            request.key = Some(key.clone());
            request.value = Some(value.clone());
        }
    }

    let request = match serde_json::to_string(&request) {
        Ok(req) => req,
        Err(e) => return Err(e.to_string()),
    };
    tracing::debug!("request to be send: {}", request);
    let start = std::time::Instant::now();

    if let Err(e) = socket.send(Message::Text(request)).await {
        return Err(e.to_string());
    }

    let msg = match socket.next().await {
        Some(response) => match response {
            Ok(msg) => msg,
            Err(e) => return Err(e.to_string()),
        },
        None => return Err("no response from the server".to_string()),
    };

    let elapsed = start.elapsed();
    tracing::debug!("request executed in {:?}", elapsed);

    match msg {
        Message::Text(json) => {
            let response: crate::common::websocket::WsResponse =
                match serde_json::from_str(&json[..]) {
                    Ok(resp) => resp,
                    Err(e) => return Err(e.to_string()),
                };

            if response.status == crate::common::websocket::WsResponseStatus::Ok {
                Ok(response.message)
            } else {
                Err(response.message)
            }
        }
        resp => Err(format!("server responded with {}", resp)),
    }
}

use serde::Deserialize;

/// Represent one node in client config toml file
#[derive(Deserialize, Debug)]
pub struct Node {
    /// Name of the node
    pub name: String,

    /// Address of the node
    pub address: String,
}

/// Represent the whole client config toml file
#[derive(Deserialize)]
pub struct Config {
    /// Vector about defined nodes
    pub node: Vec<Node>,
}

/// Read config from client's config file
///
/// # Parameters
/// - `path`: Location of client configuration file
///
/// # Return
///
/// If it found and parsed file successfully then return with config. Else return with an error text.
pub fn get_config(path: &String) -> Result<Config, String> {
    let file_content = get_file_content(path)?;

    let config: Config = match toml::from_str(&file_content[..]) {
        Ok(config) => config,
        Err(e) => return Err(format!("Failed to parse config file: {}", e)),
    };

    Ok(config)
}

/// Read a file content
///
/// # Parameters
/// - `path`: Path of file that has to be read
///
/// # Return
/// With the file content, else with an error text.
fn get_file_content(path: &String) -> Result<String, String> {
    let path = std::path::Path::new(path);
    match path.exists() {
        true => match std::fs::read_to_string(path) {
            Ok(content) => Ok(content),
            Err(e) => Err(format!(
                "File '{}' could not been read: {}",
                path.display(),
                e
            )),
        },
        false => Err(format!("File '{}' does not exist", path.display())),
    }
}

/// Read and parse client configuration
///
/// # Parameters
/// - `path`: File path where the config can be located
///
/// # Return
///
/// With a parsed config struct, else with an error text
pub fn client_config_parse(path: &String) -> Result<Config, String> {
    if std::path::Path::new(path).exists() {
        match get_config(path) {
            Ok(cfg) => return Ok(cfg),
            Err(e) => return Err(e),
        }
    }

    Err(String::from("config file does not exist"))
}

/// Get address for specified client
///
/// # Parameters
/// - `name`: Name of the server
/// - `config`: Parsed client configuration file
///
/// # Return
///
/// If found return with address, else with None.
pub fn get_address_for_client(name: String, config: &Config) -> Option<String> {
    let name = if let Some(name) = name.strip_prefix("cfg://") {
        tracing::trace!("selected config is: {}", name);
        name.to_string()
    } else {
        name
    };

    for node in &config.node {
        if node.name == name {
            tracing::debug!("found address: {}", node.address);
            return Some(node.address.clone());
        }
    }

    None
}
