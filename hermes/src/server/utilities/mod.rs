pub mod config_parse;
pub mod initial_parse;
pub mod lua;

/// Check that file exists, then return with its contant.
/// 
/// # Parameters
/// - `path`: File that must be read
/// 
/// # Return
/// 
/// With file content, or with the error text.
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
