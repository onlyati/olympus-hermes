#![allow(dead_code)]
use std::sync::{Arc, RwLock};

use crate::Database;

/// ## Parse request
/// 
/// This function parse the request which is coming on regular port of the program.
/// 
/// ### Return:
/// In case of success or error, return string begin with `>Done`. Else it begin with `>Error`.
/// Output or reply content will coming after a line break.
pub fn parse_db_command(command: &str, db: Arc<RwLock<Database>>) -> Result<String, String> {
    let temp_cmds: Vec<&str> = command.split(";").collect();
    let command = temp_cmds[0];
    let command = command.trim();

    let command_vec: Vec<&str> = command.split_whitespace().collect();

    if command_vec.len() < 3 {
        if command == "help" {
            // Help on those, who are asking
            let response = String::from(concat!(
                "Set or update key:    set data('<key>', '<value>') in <table>;\n",
                "Get value of key:     get key('<key>') in <table>;\n",
                "Delete pair:          delete key('<key>') in <table>;\n",
                "Mask keys:            keys mask('<mask>') in <table>;\n",
            ));
            return Ok(response);
        }
        if command == "version" {
            return Ok(format!("{}", crate::VERSION));
        }
        return Err(String::from("Invalid request"));
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Set key-value in table                                                                    */
    /*-------------------------------------------------------------------------------------------*/
    if command.starts_with("set data('") {
        let (key, value, db_table) = match parse_request(command, ReadValue::KeyValue) {
            Ok(v) => v,
            Err(e) => return Err(format!("{}\n", e)),
        };

        let db = db.read().unwrap();
        match db.select_table(db_table) {
            Some(table) => {
                table.insert_or_update(&key[..], &value[..]);
                return Ok(String::from("\n"));
            },
            None => return Err(format!("Table does not exist: '{}'\n", db_table)),
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Get value of specified key                                                                */
    /*-------------------------------------------------------------------------------------------*/
    if command.starts_with("get key('") {
        let (key, _, db_table) = match parse_request(command, ReadValue::Key) {
            Ok(v) => v,
            Err(e) => return Err(format!("{}\n", e)),
        };

        let db = db.read().unwrap();
        match db.select_table(db_table) {
            Some(table) => {
                match table.get_value(&key[..]) {
                    Some(value) => {
                        return Ok(format!("{}\n", value));
                    },
                    None => return Err(format!("Key '{}' does not exist in '{}' table\n", key, db_table)),
                }
                
            },
            None => return Err(format!("Table does not exist: '{}'\n", db_table)),
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Delete specified key                                                                      */
    /*-------------------------------------------------------------------------------------------*/
    if command.starts_with("delete key('") {
        let (key, _, db_table) = match parse_request(command, ReadValue::Key) {
            Ok(v) => v,
            Err(e) => return Err(format!("{}\n", e)),
        };

        let db = db.read().unwrap();
        match db.select_table(db_table) {
            Some(table) => {
                match table.remove_key(&key[..]) {
                    Some(_) => {
                        return Ok(String::from("\n"));
                    },
                    None => return Err(format!("Key '{}' does not exist in '{}' table\n", key, db_table)),
                }
                
            },
            None => return Err(format!("Table does not exist: '{}'\n", db_table)),
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* List keys                                                                                 */
    /*-------------------------------------------------------------------------------------------*/
    if command.starts_with("keys mask('") {
        let (key, _, db_table) = match parse_request(command, ReadValue::Key) {
            Ok(v) => v,
            Err(e) => return Err(format!("{}\n", e)),
        };

        let db = db.read().unwrap();
        match db.select_table(db_table) {
            Some(table) => {
                let mut response = String::from("\n");
                for key in table.key_start_with(&key[..]) {
                    response += &key[..];
                    response += "\n";
                }
                return Ok(response);
            },
            None => return Err(format!("Table does not exist: '{}'\n", db_table)),
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Create table requests                                                                     */
    /*-------------------------------------------------------------------------------------------*/
    if command.starts_with("create table") {
        let mut db = db.write().unwrap();
        match db.create_table(String::from(command_vec[2])) {
            Ok(_) => return Ok(String::from("\n")),
            Err(e) => return Err(format!("{}\n", e)),
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Drop table requests                                                                       */
    /*-------------------------------------------------------------------------------------------*/
    if command.starts_with("drop table") {
        let mut db = db.write().unwrap();
        match db.drop_table(command_vec[2]) {
            Ok(_) => return Ok(String::from("\n")),
            Err(e) => return Err(format!("{}\n", e)),
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* List all tables                                                                           */
    /*-------------------------------------------------------------------------------------------*/
    if command == "list all tables" {
        let db = db.read().unwrap();
        let tables = db.get_tables();
        let mut list = String::from("");
        for table in tables {
            list += table.get_name();
            list += "\n";
        }
        return Ok(list);
    }

    return Err(format!("Invalid request: {}", command_vec[0]));
}

#[derive(PartialEq, Eq)]
enum ReadMode {
    WaitForKey,
    ReadKey,
    WaitForValue,
    ReadValue,
}

#[derive(PartialEq, Eq)]
enum ReadValue {
    Key,
    KeyValue,
}

/// ## Parse key, value and table name from request
/// 
/// Read values accordingly `ReadValue::Key`, `ReadValue::KeyValue`˙was specified.
fn parse_request(request: &str, read_what: ReadValue) -> Result<(String, String, &str), String> {
    let mut key = String::new();
    let mut value = String::new();
    let mut what = ReadMode::WaitForKey;

    for c in request.chars() {
        if c == '\''  {
            if what == ReadMode::WaitForKey {
                what = ReadMode::ReadKey;
            }
            else if what == ReadMode::ReadKey {
                what = ReadMode::WaitForValue;
                if read_what == ReadValue::Key {
                    break;
                }
            }
            else if what == ReadMode::WaitForValue && read_what == ReadValue::KeyValue {
                what = ReadMode::ReadValue;
            }
            else if what == ReadMode::ReadValue && read_what == ReadValue:: KeyValue {
                break;
            }

            continue;
        }

        if what == ReadMode::ReadKey {
            key += c.to_string().as_str();
            continue;
        }

        if what == ReadMode::ReadValue {
            value += c.to_string().as_str();
            continue;
        }
    }

    let req_vec: Vec<&str> = request.split_whitespace().collect();

    let word_num = req_vec.len();
    if req_vec[word_num - 2] != "in" {
        return Err(format!("Parse error nearby '{}'\nSyntax is: set data('<key>', '<value>') in <table>;", req_vec[word_num - 2]));
    }

    let table = req_vec[word_num - 1];

    return Ok((key, value, table));
}