use serde::Deserialize;

#[derive(Deserialize)]
pub struct Node {
    pub name: String,
    pub address: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub node: Vec<Node>,
}

/// Read config from client's config file
pub fn get_config(path: &String) -> Result<Config, String> {
    let file_content = get_file_content(path)?;

    let config: Config = match toml::from_str(&file_content[..]) {
        Ok(config) => config,
        Err(e) => return Err(format!("Failed to parse config file: {}", e)),
    };

    return Ok(config);
}

/// Read a file content
fn get_file_content(path: &String) -> Result<String, String> {
    let path = std::path::Path::new(path);
    match path.exists() {
        true => match std::fs::read_to_string(path) {
            Ok(content) => return Ok(content),
            Err(e) => {
                return Err(format!(
                    "File '{}' could not been read: {}",
                    path.display(),
                    e
                ))
            }
        },
        false => return Err(format!("File '{}' does not exist", path.display())),
    }
}
