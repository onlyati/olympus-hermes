use serde::Deserialize;

/// Represent a network table in config toml file
///
/// # Example:
/// ```toml
/// [network]
/// classic = "127.0.0.1:3031"     # Classic TCP interface bind to this address
/// rest = "0.0.0.0:3032"          # REST interface bind to this address
/// websocket = "127.0.0.1:3033"   # Websocket interface bind to this address
/// ```
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Network {
    pub classic: Option<String>,
    pub rest: Option<String>,
    pub websocket: Option<String>,
}

/// Represent a initials table in config toml file
///
/// # Example
/// ```toml
/// [initials]
/// # Records and hooks will be read from here during startup
/// path = "/home/ati/work/OnlyAti.Hermes/hermes/init_data.toml"
/// ```
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Initials {
    pub path: String,
}

/// Represent a logger table in config toml file
///
/// # Example
/// ```toml
/// [logger]
/// location = "/tmp/hermes-datastore-test.txt" # Which file should the database log written
/// ```
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Logger {
    pub mem_only: bool,
    pub location: Option<String>,
}

/// Represent a scripts table in config toml file
///
/// # Example
/// ```toml
/// [scripts]
/// lib_path = "./lua-examples/libs"
/// exec_path = "./lua-examples"
/// execs = [
///     "test.lua",
///     "work_with_words.lua",
///     "simple_words.lua",
///     "error_example.lua",
/// ]
/// ```
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Scripts {
    pub lib_path: Option<String>,
    pub exec_path: String,
    pub execs: Vec<String>,
}

/// Represent a gitea table in config toml file
///
/// # Example
/// ```toml
/// [gitea]
/// enable = true
/// script = "gitea_parser.lua"
/// key_base = "/root/gitea"
/// ```
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Gitea {
    pub enable: bool,
    pub script: String,
    pub key_base: String,
}

/// Represent the whole config.toml file
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Config {
    pub network: Network,
    pub initials: Initials,
    pub logger: Logger,
    pub scripts: Option<Scripts>,
    pub gitea: Option<Gitea>,
}

/// This function parse the passed toml config file and create a struct based on it.
///
/// # Parameters
/// - `config_path`: A path to toml config file
///
/// # Return
///
/// With Ok(Config) if parse failed, else with Err(String) that describes the problem.
pub fn parse_config(config_path: &String) -> Result<Config, String> {
    // Read the file and doing the parse
    let file_content = super::get_file_content(config_path)?;

    let mut config: Config = match toml::from_str(&file_content[..]) {
        Ok(config) => config,
        Err(e) => return Err(format!("Failed to parse config file: {}", e)),
    };

    // If nothing to listen, then no reason to start application
    if config.network.classic.is_none()
        && config.network.rest.is_none()
        && config.network.websocket.is_none()
    {
        return Err(String::from("At least one interface must be enabled"));
    }

    // Write out the config items
    tracing::info!("Config settings:");
    tracing::info!("- network.classic: {:?}", config.network.classic);
    tracing::info!("- network.rest: {:?}", config.network.rest);
    tracing::info!("- network.websocket: {:?}", config.network.websocket);
    tracing::info!("- initials.path: {}", config.initials.path);
    tracing::info!("- logger.mem_only: {}", config.logger.mem_only);

    if !config.logger.mem_only {
        if let Some(location) = &config.logger.location {
            tracing::info!("- logger.location: {}", location);
        } else {
            return Err("parameter mem_only is true but no logger location defined".to_string());
        }
    }

    // If there are scripts for EXEC endpoint then display its settings
    if let Some(scripts) = &mut config.scripts {
        if let Some(lib_path) = &scripts.lib_path {
            tracing::info!("- scripts.lib_path: {}", lib_path);
            std::env::set_var("LUA_PATH", format!("{}/?.lua;;", lib_path));
        }
        tracing::info!("- scripts.exec_path: {}", scripts.exec_path);

        let mut to_remove = vec![];

        for i in 0..scripts.execs.len() {
            let path = format!("{}/{}", scripts.exec_path, scripts.execs[i]);
            if !std::path::Path::new(&path).exists() {
                to_remove.push(i);
            }
        }

        to_remove.reverse();

        for i in to_remove {
            tracing::error!(
                "script '{}' does not exist, removed from execs in config",
                scripts.execs[i]
            );
            scripts.execs.remove(i);
        }

        tracing::debug!("- scripts.execs: {:?}", scripts.execs);
    }

    // If Gitea plugin is active display its settings
    if let Some(gitea) = &config.gitea {
        tracing::info!("- gitea.enable: {}", gitea.enable);
        tracing::info!("- gitea.script: {}", gitea.script);
        tracing::info!("- gitea.key_base: {}", gitea.key_base);
    }

    Ok(config)
}
