use serde::Deserialize;

/// Represent a network table in initial toml file
#[derive(Deserialize)]
pub struct Network {
    pub classic: Option<String>,
    pub grpc: Option<String>,
    pub rest: Option<String>,
}

/// Represent a initials table in initial toml file
#[derive(Deserialize)]
pub struct Initials {
    pub path: String,
}

/// Represent a logger table in initial toml file
#[derive(Deserialize)]
pub struct Logger {
    pub location: String,
}

/// Represent the whole config.toml file
#[derive(Deserialize)]
pub struct Config {
    pub network: Network,
    pub initials: Initials,
    pub logger: Logger,
}

pub fn parse_config(config_path: &String) -> Result<Config, String> {
    let file_content = super::get_file_content(config_path)?;

    let config: Config = match toml::from_str(&file_content[..]) {
        Ok(config) => config,
        Err(e) => return Err(format!("Failed to parse config file: {}", e)),
    };

    tracing::info!("Config settings:");
    tracing::info!("- network.classic: {:?}", config.network.classic);
    tracing::info!("- network.grpc: {:?}", config.network.grpc);
    tracing::info!("- network.rest: {:?}", config.network.rest);
    tracing::info!("- initials.path: {}", config.initials.path);
    tracing::info!("- logger.location: {}", config.logger.location);

    return Ok(config);
}