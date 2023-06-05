use serde::Deserialize;

/// Represent a network table in initial toml file
#[derive(Deserialize, Clone)]
pub struct Network {
    pub classic: Option<String>,
    pub grpc: Option<String>,
    pub rest: Option<String>,
}

/// Represent a initials table in initial toml file
#[derive(Deserialize, Clone)]
pub struct Initials {
    pub path: String,
}

/// Represent a logger table in initial toml file
#[derive(Deserialize, Clone)]
pub struct Logger {
    pub location: String,
}

/// Represent a scripts table in initial toml file
#[derive(Deserialize, Clone)]
pub struct Scripts {
    pub lib_path: Option<String>,
    pub exec_path: String,
    pub execs: Vec<String>,
}

/// Represent the whole config.toml file
#[derive(Deserialize, Clone)]
pub struct Config {
    pub network: Network,
    pub initials: Initials,
    pub logger: Logger,
    pub scripts: Option<Scripts>,
}

pub fn parse_config(config_path: &String) -> Result<Config, String> {
    let file_content = super::get_file_content(config_path)?;

    let mut config: Config = match toml::from_str(&file_content[..]) {
        Ok(config) => config,
        Err(e) => return Err(format!("Failed to parse config file: {}", e)),
    };

    if config.network.classic.is_none()
        && config.network.grpc.is_none()
        && config.network.rest.is_none()
    {
        return Err(String::from("At least one interface must be enabled"));
    }

    tracing::info!("Config settings:");
    tracing::info!("- network.classic: {:?}", config.network.classic);
    tracing::info!("- network.grpc: {:?}", config.network.grpc);
    tracing::info!("- network.rest: {:?}", config.network.rest);
    tracing::info!("- initials.path: {}", config.initials.path);
    tracing::info!("- logger.location: {}", config.logger.location);

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
            tracing::error!("script '{}' does not exist, removed from execs in config", scripts.execs[i]);
            scripts.execs.remove(i);
        }

        tracing::debug!("- scripts.execs: {:?}", scripts.execs);
    }

    return Ok(config);
}
