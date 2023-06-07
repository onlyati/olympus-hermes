use mlua::{Lua, Table};

use super::config_parse::{Scripts};

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
            return Ok(pair);
        },
        Err(e) => {
            tracing::error!("failed to execute {} script", command);
            return Err(e.to_string());
        },
    }
}

async fn run_lua(
    old_pair: Option<(String, String)>,
    new_pair: (String, String),
    params: Option<String>,
    script: String
) -> Result<(String, String), mlua::Error> {
    tracing::trace!("initializing lua environment");
    let lua = Lua::new();
    let globals = lua.globals();

    if let Some(old) = old_pair {
        tracing::trace!("key-value pair already existed, set them as global");
        let old_table = lua.create_table()?;
        old_table.set("key", old.0.clone())?;
        old_table.set("value", old.1.clone())?;
        globals.set("old", old_table)?;
    }

    tracing::trace!("set new key-value pair as global");
    let new_table = lua.create_table()?;
    new_table.set("key", new_pair.0.clone())?;
    new_table.set("value", new_pair.1.clone())?;

    if let Some(params) = params.clone() {
        tracing::trace!("set parameter as global");
        new_table.set("parm", params)?;
    }

    globals.set("new", new_table)?;

    tracing::debug!("execute '{}' lua script for '{}' key with '{:?}' parameter", script, new_pair.0, params);
    lua.load(std::path::Path::new(&script)).exec()?;

    tracing::trace!("read modified new value and key from lua environment");
    let final_key: Table = globals.get("new")?;

    let final_value = final_key.get("value")?;
    let final_key = final_key.get("key")?;

    return Ok((final_key, final_value));
}
