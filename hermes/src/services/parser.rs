#![allow(dead_code)]
use std::sync::{Arc, RwLock};

use crate::Database;

pub fn parse_db_command(command: &str, db: Arc<RwLock<Database>>) -> Result<String, String> {
    let temp_cmds: Vec<&str> = command.split(";").collect();
    let command = temp_cmds[0];

    let command_vec: Vec<&str> = command.split_whitespace().collect();

    println!("[{}]", command);

    if command_vec.len() < 3 {
        return Err(String::from("Invalid request"));
    }

    if command.starts_with("create table") {
        let mut db = db.write().unwrap();
        match db.create_table(String::from(command_vec[2])) {
            Ok(_) => return Ok(String::from(">Done\n")),
            Err(e) => return Err(format!(">Error\n{}\n", e)),
        }
    }

    if command.starts_with("drop table") {
        let mut db = db.write().unwrap();
        match db.drop_table(command_vec[2]) {
            Ok(_) => return Ok(String::from(">Done\n")),
            Err(e) => return Err(format!(">Error\n{}\n", e)),
        }
    }

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

    if command.starts_with("set data(") {

    }

    if command.starts_with("get key(") {

    }

    if command.starts_with("delete key(") {

    }

    return Err(format!("Invalid request: {}", command_vec[0]));
}