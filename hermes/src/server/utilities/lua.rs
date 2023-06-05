use mlua::{Lua, Table};

use super::config_parse::{Scripts};

pub async fn run(
    config: Scripts,
    old_pair: Option<(String, String)>,
    new_pair: (String, String),
    command: String,
    params: Option<String>,
) -> Result<(String, String), String> {
    match run_lua(old_pair, new_pair, params, format!("{}/{}", config.exec_path, command)).await {
        Ok(pair) => return Ok(pair),
        Err(e) => return Err(e.to_string()),
    }
}

async fn run_lua(
    old_pair: Option<(String, String)>,
    new_pair: (String, String),
    params: Option<String>,
    script: String
) -> Result<(String, String), mlua::Error> {
    let lua = Lua::new();
    let globals = lua.globals();

    if let Some(old) = old_pair {
        let old_table = lua.create_table()?;
        old_table.set("key", old.0.clone())?;
        old_table.set("value", old.1.clone())?;
        globals.set("old", old_table)?;
    }

    let new_table = lua.create_table()?;
    new_table.set("key", new_pair.0.clone())?;
    new_table.set("value", new_pair.1.clone())?;

    if let Some(params) = params {
        new_table.set("parm", params.clone())?;
    }

    globals.set("new", new_table)?;

    lua.load(std::path::Path::new(&script)).exec()?;

    let final_key: Table = globals.get("new")?;

    let final_value = final_key.get("value")?;
    let final_key = final_key.get("key")?;

    return Ok((final_key, final_value));
}
