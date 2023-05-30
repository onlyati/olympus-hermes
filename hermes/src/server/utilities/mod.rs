pub mod initial_parse;
pub mod config_parse;


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