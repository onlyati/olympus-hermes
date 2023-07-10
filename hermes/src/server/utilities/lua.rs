use mlua::{Lua, Table};

use super::config_parse::{Scripts};

/// This is called from endpoint that wants to run Lua script.
/// This function forward the request the a Lua runtime environment.
/// 
/// # Parameters
/// - `config`: Script part from configuration that contains the allowed script list and their location
/// - `old_pair`: If there was entry in Hermes, then the old value is passed. This is delivered into Lua environment as global variables
/// - `new_pair`: They are the new pairs that was specified in the incoming Hermes request, this is delivered to Lua environment as global variables
/// - `command`: Script name from the incoming Hermes request
/// - `params`: Optional parameter that is delivered to Lua environment as global variables
/// 
/// # Return
/// 
/// If everything went fine, it returns with the modified new key-value pair. This will be written or been used as trigger in Hermes.
/// In case of fail, it send back the Lua error.
pub async fn run(
    config: Scripts,
    old_pair: Option<(String, String)>,
    new_pair: (String, String),
    command: String,
    params: Option<String>,
) -> Result<(String, String), String> {
    tracing::trace!("requested to run lua script: {}, parameter: {:?}, key: {}", command, params, new_pair.0);
    match run_lua(old_pair, new_pair, params, format!("{}/{}", config.exec_path, command)).await {
        Ok(pair) => {
            tracing::debug!("script {} has succesfully run", command);
            Ok(pair)
        },
        Err(e) => {
            tracing::error!("failed to execute {} script", command);
            Err(e.to_string())
        },
    }
}

/// Create a Lua runtime environment, load the global variables, run it then return with the modified pair.
/// Defined global variables that can be used in Lua script:
/// - `_G.new["key"]` and `_G.new["value"]`: The new key-value pair
/// - `_G.new["parm"]`: If paramater was defined in request, it is here else Nil
/// - `_G.old["key"]` and `_G.old["value"]`: If requested key was already existed, then it can be read from here. If it is a new entry, then it is Nil
/// 
/// # Parameters
/// - `old_pair`: If there was entry in Hermes, then the old value is passed. This is delivered into Lua environment as global variables
/// - `new_pair`: They are the new pairs that was specified in the incoming Hermes request, this is delivered to Lua environment as global variables
/// - `params`: Optional parameter that is delivered to Lua environment as global variables
/// - `script`: Script name from the incoming Hermes request
/// 
/// # Return
/// 
/// If everything went fine, it returns with the modified new key-value pair. This will be written or been used as trigger in Hermes.
/// In case of fail, it send back the Lua error.
async fn run_lua(
    old_pair: Option<(String, String)>,
    new_pair: (String, String),
    params: Option<String>,
    script: String
) -> Result<(String, String), mlua::Error> {
    // Create a Lua environment
    tracing::trace!("initializing lua environment");
    let lua = Lua::new();
    let globals = lua.globals();

    //
    // Setup the global variables
    //

    // _G.old["key"] and _G.old["value"] contains the previous value of this record. If this is a new record, then _G.old is a Nil
    if let Some(old) = old_pair {
        tracing::trace!("key-value pair already existed, set them as global");
        let old_table = lua.create_table()?;
        old_table.set("key", old.0.clone())?;
        old_table.set("value", old.1)?;
        globals.set("old", old_table)?;
    }

    // _G.new["key"] and _G.new["value"] contains the new values that has been passed to hermes in a request
    tracing::trace!("set new key-value pair as global");
    let new_table = lua.create_table()?;
    new_table.set("key", new_pair.0.clone())?;
    new_table.set("value", new_pair.1.clone())?;

    // If parameter also defined then set _G.new["parm"]. If no parameter then its value is Nil
    if let Some(params) = params.clone() {
        tracing::trace!("set parameter as global");
        new_table.set("parm", params)?;
    }

    globals.set("new", new_table)?;

    // Run the Lua script
    tracing::debug!("execute '{}' lua script for '{}' key with '{:?}' parameter", script, new_pair.0, params);
    lua.load(std::path::Path::new(&script)).exec()?;

    // Get the modified new key-value pair then return with this
    tracing::trace!("read modified new value and key from lua environment");
    let final_key: Table = globals.get("new")?;

    let final_value = final_key.get("value")?;
    let final_key = final_key.get("key")?;

    Ok((final_key, final_value))
}

/// Lua runtime for Gitea plugin, this has different inputs then the regular Lua runner function.
/// Only `_G.new["key"]` and `_G.new["value"]` is passed. The key is get from the config.toml file.
/// Value will be the whole message body that is send by Gitea. If both has value after script has
/// run, then it will be saved into Hermes.
/// 
/// # Parameters
/// - `script`: Defined Gitea plugin script from the config
/// - `body`: Whole message that is sent by Gita
/// - `key_prefix`: Defined key prefix from the config
/// 
/// # Return
/// 
/// If everything went fine, it returns with the modified new key-value pair. This will be written or been used as trigger in Hermes.
/// In case of fail, it send back the Lua error.
pub async fn run_lua_for_gitea(
    script: String,
    body: String,
    key_prefix: String
) -> Result<(String, String), mlua::Error> {
    // Allocate Lua runtime
    tracing::trace!("initializing lua environment");
    let lua = Lua::new();
    let globals = lua.globals();

    // Set the new pairs
    tracing::trace!("setup new key-value pair as globa");
    let new_table = lua.create_table()?;
    new_table.set("key", key_prefix)?;
    new_table.set("value", body)?;

    globals.set("new", new_table)?;

    // Execute script
    tracing::debug!("execute {} script for gitea endpoint", script);
    lua.load(std::path::Path::new(&script)).exec()?;

    // Get the modified key-value pair then return with this
    tracing::trace!("read modified new value and key from lua environment");
    let final_key: Table = globals.get("new")?;

    let final_value = final_key.get("value")?;
    let final_key = final_key.get("key")?;

    Ok((final_key, final_value))
}