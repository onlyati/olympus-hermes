#![allow(dead_code)]
use std::sync::{Arc, RwLock};

use crate::Database;

#[derive(PartialEq, Eq)]
enum ReadMode {
    WaitForKey,
    ReadKey,
    WaitForValue,
    ReadValue,
}

pub fn parse_db_command(command: &str, db: Arc<RwLock<Database>>) -> Result<String, String> {
    let temp_cmds: Vec<&str> = command.split(";").collect();
    let command = temp_cmds[0];
    let command = command.trim();

    let command_vec: Vec<&str> = command.split_whitespace().collect();

    if command_vec.len() < 3 {
        return Err(String::from("Invalid request"));
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Set key-value in table                                                                    */
    /*-------------------------------------------------------------------------------------------*/
    if command.starts_with("set data('") {
        let mut key = String::new();
        let mut value = String::new();
        let mut what = ReadMode::WaitForKey;

        for c in command.chars() {
            if c == '\''  {
                if what == ReadMode::WaitForKey {
                    what = ReadMode::ReadKey;
                }
                else if what == ReadMode::ReadKey {
                    what = ReadMode::WaitForValue;
                }
                else if what == ReadMode::WaitForValue {
                    what = ReadMode::ReadValue;
                }
                else if what == ReadMode::ReadValue {
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

        let word_num = command_vec.len();
        if command_vec[word_num - 2] != "in" {
            return Err(format!(">Error\nParse error nearby '{}'\nSyntax is: set data('<key>', '<value>') in <table>;", command_vec[word_num - 2]))
        }

        let db_table = command_vec[word_num - 1];

        let db = db.read().unwrap();
        match db.select_table(db_table) {
            Some(table) => {
                table.insert_or_update(&key[..], &value[..]);
                return Ok(String::from(">Done\n"));
            },
            None => return Err(format!(">Error\nTable does not exist: '{}'\n", db_table)),
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Get value of specified key                                                                */
    /*-------------------------------------------------------------------------------------------*/
    if command.starts_with("get key('") {
        let mut key = String::new();
        let mut what = ReadMode::WaitForKey;

        for c in command.chars() {
            if c == '\''  {
                if what == ReadMode::WaitForKey {
                    what = ReadMode::ReadKey;
                }
                else if what == ReadMode::ReadKey {
                    break;
                }

                continue;
            }

            if what == ReadMode::ReadKey {
                key += c.to_string().as_str();
                continue;
            }
        }

        let word_num = command_vec.len();
        if command_vec[word_num - 2] != "in" {
            return Err(format!(">Error\nParse error nearby '{}'\nSyntax is: get key('<key>') in <table>;", command_vec[word_num - 2]))
        }

        let db_table = command_vec[word_num - 1];

        let db = db.read().unwrap();
        match db.select_table(db_table) {
            Some(table) => {
                match table.get_value(&key[..]) {
                    Some(value) => {
                        
                        return Ok(format!(">Done\n{}\n", value));
                    },
                    None => return Err(format!(">Error\nKey '{}' does not exist in '{}' table\n", key, db_table)),
                }
                
            },
            None => return Err(format!(">Error\nTable does not exist: '{}'\n", db_table)),
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Delete specified key                                                                      */
    /*-------------------------------------------------------------------------------------------*/
    if command.starts_with("delete key('") {
        let mut key = String::new();
        let mut what = ReadMode::WaitForKey;

        for c in command.chars() {
            if c == '\''  {
                if what == ReadMode::WaitForKey {
                    what = ReadMode::ReadKey;
                }
                else if what == ReadMode::ReadKey {
                    break;
                }

                continue;
            }

            if what == ReadMode::ReadKey {
                key += c.to_string().as_str();
                continue;
            }
        }

        let word_num = command_vec.len();
        if command_vec[word_num - 2] != "in" {
            return Err(format!(">Error\nParse error nearby '{}'\nSyntax is: delete key('<key>') in <table>;", command_vec[word_num - 2]))
        }

        let db_table = command_vec[word_num - 1];

        let db = db.read().unwrap();
        match db.select_table(db_table) {
            Some(table) => {
                match table.remove_key(&key[..]) {
                    Some(_) => {
                        return Ok(String::from(">Done\n"));
                    },
                    None => return Err(format!(">Error\nKey '{}' does not exist in '{}' table\n", key, db_table)),
                }
                
            },
            None => return Err(format!(">Error\nTable does not exist: '{}'\n", db_table)),
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* List keys                                                                                 */
    /*-------------------------------------------------------------------------------------------*/
    if command.starts_with("keys mask('") {
        let mut key = String::new();
        let mut what = ReadMode::WaitForKey;

        for c in command.chars() {
            if c == '\''  {
                if what == ReadMode::WaitForKey {
                    what = ReadMode::ReadKey;
                }
                else if what == ReadMode::ReadKey {
                    break;
                }

                continue;
            }

            if what == ReadMode::ReadKey {
                key += c.to_string().as_str();
                continue;
            }
        }

        let word_num = command_vec.len();
        if command_vec[word_num - 2] != "in" {
            return Err(format!(">Error\nParse error nearby '{}'\nSyntax is: keys mask('<mask-for-begin>') in <table>;", command_vec[word_num - 2]))
        }

        let db_table = command_vec[word_num - 1];

        let db = db.read().unwrap();
        match db.select_table(db_table) {
            Some(table) => {
                let mut response = String::from(">Done\n");
                for key in table.key_start_with(&key[..]) {
                    response += &key[..];
                    response += "\n";
                }
                return Ok(response);
            },
            None => return Err(format!(">Error\nTable does not exist: '{}'\n", db_table)),
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Create table requests                                                                     */
    /*-------------------------------------------------------------------------------------------*/
    if command.starts_with("create table") {
        let mut db = db.write().unwrap();
        match db.create_table(String::from(command_vec[2])) {
            Ok(_) => return Ok(String::from(">Done\n")),
            Err(e) => return Err(format!(">Error\n{}\n", e)),
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Drop table requests                                                                       */
    /*-------------------------------------------------------------------------------------------*/
    if command.starts_with("drop table") {
        let mut db = db.write().unwrap();
        match db.drop_table(command_vec[2]) {
            Ok(_) => return Ok(String::from(">Done\n")),
            Err(e) => return Err(format!(">Error\n{}\n", e)),
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* List all tables                                                                           */
    /*-------------------------------------------------------------------------------------------*/
    if command == "list all tables" {
        let db = db.read().unwrap();
        let tables = db.get_tables();
        let mut list = String::from(">Done\n");
        for table in tables {
            list += table.get_name();
            list += "\n";
        }
        return Ok(list);
    }

    return Err(format!("Invalid request: {}", command_vec[0]));
}