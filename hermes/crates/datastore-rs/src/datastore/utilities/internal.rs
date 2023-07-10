use super::{
    Table, {ErrorKind, KeyType, ListType, ValueType},
};

/// Validate and parse the key string.
/// For example: /root/status/sub1 -> ["root", "status", "sub1"]
pub(crate) fn validate_key<'a>(
    key_string: &'a str,
    db_name: &String,
) -> Result<Vec<&'a str>, ErrorKind> {
    if &key_string[0..1] != "/" {
        return Err(ErrorKind::InvalidKey(
            "Key must begin with '/' sign".to_string(),
        ));
    }

    let key_routes = key_string
        .split('/')
        .filter(|x| !x.is_empty())
        .collect::<Vec<&str>>();

    if key_routes.is_empty() {
        return Err(ErrorKind::InvalidKey(
            "Key must contain at least 1 items, e.g.: /root/status".to_string(),
        ));
    }

    if key_routes[0] != db_name {
        return Err(ErrorKind::InvalidKey(
            "Key does not begin with the root table".to_string(),
        ));
    }

    Ok(key_routes)
}

/// Recursive algoritm to find a table
pub(crate) fn find_table<'a>(db: &'a Table, routes: Vec<&'a str>) -> Option<&'a Table> {
    if routes.is_empty() {
        return Some(db);
    }

    let current_table = KeyType::Table(routes[0].to_string());
    match db.get(&current_table) {
        Some(ValueType::TablePointer(table_pointer)) => {
            return find_table(table_pointer, routes[1..].to_vec());
        }
        _ => None,
    }
}

/// Recursive algoritm the find a table and return as mutable reference
pub(crate) fn find_table_mut<'a>(db: &'a mut Table, routes: Vec<&'a str>) -> Option<&'a mut Table> {
    if routes.is_empty() {
        return Some(db);
    }

    let current_table = KeyType::Table(routes[0].to_string());
    match db.get_mut(&current_table) {
        Some(ValueType::TablePointer(table_pointer)) => {
            return find_table_mut(table_pointer, routes[1..].to_vec());
        }
        _ => None,
    }
}

/// Display all items from a table
pub(crate) fn display_tables(
    db: &Table,
    key_prefix: &String,
    level: &ListType,
) -> Result<Vec<KeyType>, ErrorKind> {
    let mut result: Vec<KeyType> = Vec::with_capacity(std::mem::size_of::<KeyType>() * db.len());

    for (key, value) in db.iter() {
        match key {
            KeyType::Record(key) => {
                let new_key = format!("{}/{}", key_prefix.clone(), key);
                let new_key = KeyType::Record(new_key);
                result.push(new_key);
            }
            KeyType::Table(key) => {
                if *level == ListType::OneLevel {
                    continue;
                }

                let table_name = match value {
                    ValueType::TablePointer(table) => table,
                    _ => continue,
                };
                let mut temp = display_tables(
                    table_name,
                    &format!("{}/{}", key_prefix, key),
                    level,
                )?;

                result.append(&mut temp);
            }
            KeyType::Queue(key) => {
                let new_key = format!("{}/{}", key_prefix.clone(), key);
                let new_key = KeyType::Queue(new_key);
                result.push(new_key);
            }
        }
    }

    Ok(result)
}
