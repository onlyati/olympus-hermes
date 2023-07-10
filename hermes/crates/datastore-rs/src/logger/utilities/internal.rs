use std::path::Path;

use crate::logger::LogItem;

/// Read append file and deserialize it into a vector.
///
/// # Arguments
/// 1. `path`: Append file that has to bread and deserialize
///
/// # Return
///
/// If everything went fine then return with the vector. Else with an error message.
/// If the logging is not enabled it returns with an empty vector.
pub fn read_append_file(path: &Path) -> Result<Vec<LogItem>, String> {
    let mut rows = Vec::new();

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = match std::fs::read(path) {
        Ok(content) => content,
        Err(e) => return Err(e.to_string()),
    };

    let mut vector_index = 0;

    while vector_index < content.len() {
        let item: LogItem = match bincode::deserialize(&content[vector_index..]) {
            Ok(item) => item,
            Err(e) => return Err(e.to_string()),
        };

        let encoded_item = match bincode::serialize(&item) {
            Ok(encoded) => encoded,
            Err(e) => return Err(e.to_string()),
        };

        rows.push(item);
        vector_index += encoded_item.len();
    }

    Ok(rows)
}
