#![allow(dead_code)]

use crate::DB;
use crate::AGENTS;

use super::agent::AgentStatus;

/// ## Parse request
/// 
/// This function parse the request which is coming on regular port of the program.
/// 
/// ### Return:
/// In case of success or error, return string begin with `>Done`. Else it begin with `>Error`.
/// Output or reply content will coming after a line break.
pub fn parse_db_command(command: &str) -> Result<String, String> {
    let temp_cmds: Vec<&str> = command.split(";").collect();
    let command = temp_cmds[0];
    let command = command.trim();

    let command_vec: Vec<&str> = command.split_whitespace().collect();

    if command_vec.len() < 2 {
        if command == "help" {
            // Help on those, who are asking
            let response = String::from(concat!(
                "Set or update key:    set data('<key>', '<value>') in <table>;\n",
                "Get value of key:     get key('<key>') in <table>;\n",
                "Delete pair:          delete key('<key>') in <table>;\n",
                "Mask keys:            keys mask('<mask>') in <table>;\n",
                "Running agent list:   agent list\n",
                "Agent details:        agent get <id>\n",
                "Enable agent:         agent allow <id>\n",
                "Disable agent:        agent forbid <id>\n",
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

        let db = DB.read().unwrap();
        if let Some(db) = &*db {
            match db.select_table(db_table) {
                Some(table) => {
                    table.insert_or_update(&key[..], &value[..]);
                    return Ok(String::from("\n"));
                },
                None => return Err(format!("Table does not exist: '{}'\n", db_table)),
            }
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

        let db = DB.read().unwrap();
        if let Some(db) = &*db {
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
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Delete specified key                                                                      */
    /*-------------------------------------------------------------------------------------------*/
    if command.starts_with("delete key('") {
        let (key, _, db_table) = match parse_request(command, ReadValue::Key) {
            Ok(v) => v,
            Err(e) => return Err(format!("{}\n", e)),
        };

        let db = DB.read().unwrap();
        if let Some(db) = &*db {
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
    }

    /*-------------------------------------------------------------------------------------------*/
    /* List keys                                                                                 */
    /*-------------------------------------------------------------------------------------------*/
    if command.starts_with("keys mask('") {
        let (key, _, db_table) = match parse_request(command, ReadValue::Key) {
            Ok(v) => v,
            Err(e) => return Err(format!("{}\n", e)),
        };

        let db = DB.read().unwrap();
        if let Some(db) = &*db {
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
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Create table requests                                                                     */
    /*-------------------------------------------------------------------------------------------*/
    if command.starts_with("create table") {
        let mut db = DB.write().unwrap();
        if let Some(db) = &mut *db {
            match db.create_table(String::from(command_vec[2])) {
                Ok(_) => return Ok(String::from("\n")),
                Err(e) => return Err(format!("{}\n", e)),
            }
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Drop table requests                                                                       */
    /*-------------------------------------------------------------------------------------------*/
    if command.starts_with("drop table") {
        let mut db = DB.write().unwrap();
        if let Some(db) = &mut *db {
            match db.drop_table(command_vec[2]) {
                Ok(_) => return Ok(String::from("\n")),
                Err(e) => return Err(format!("{}\n", e)),
            }
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* List all tables                                                                           */
    /*-------------------------------------------------------------------------------------------*/
    if command == "list all tables" {
        let db = DB.read().unwrap();
        if let Some(db) = &*db {
            let tables = db.get_tables();
            let mut list = String::from("");
            for table in tables {
                list += table.get_name();
                list += "\n";
            }
            return Ok(list);
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Handle agents                                                                             */
    /*-------------------------------------------------------------------------------------------*/
    if command_vec[0] == "agent" {

        if command_vec.len() == 2 {
            if command_vec[1] == "list" {
                let agents = AGENTS.read().unwrap();
                let agents = match &*agents {
                    Some(agents) => agents,
                    None => return Ok(String::from("\n")),
                };
                
                if agents.len() == 0 {
                    return Ok(String::from("\n"));
                }
                else {
                    let mut agent_list = String::new();
                    for agent in agents.iter() {
                        agent_list += &agent.0[..];
                        agent_list += "\n";
                    }
                    return Ok(agent_list);
                }
            }
        }

        if command_vec.len() == 3 {
            if command_vec[1] == "get" {
                let agents = AGENTS.read().unwrap();
                let agents = match &*agents {
                    Some(agents) => agents,
                    None => return Err(String::from("No agents were found\n")),
                };

                match agents.get(command_vec[2]) {
                    Some(agent) => {
                        let lr = match agent.get_last_run() {
                            Some(s) => String::from(s),
                            None => String::from("Never"),
                        };
                        return Ok(format!("id: {}\nStatus: {}\nLast run: {}\nInterval: {:?}\n", agent.get_id(), agent.get_status(), lr, agent.get_interval()));
                    },
                    None => return Err(String::from("No agent were found\n")),
                }
            }
            else if command_vec[1] == "allow" {
                let mut agents = AGENTS.write().unwrap();
                let agents = match &mut *agents {
                    Some(agents) => agents,
                    None => return Err(String::from("No agents were found\n")),
                };

                match agents.get_mut(command_vec[2]) {
                    Some(agent) => {
                        agent.put_ready();
                        if *agent.get_status() == AgentStatus::Ready{
                            return Ok(String::from("\n"));
                        }
                        else {
                            return Err(String::from("Failed to move agent into ready\n"));
                        }
                    },
                    None => return Err(String::from("No agent were found\n")),
                }
            }
            else if command_vec[1] == "forbid" {
                let mut agents = AGENTS.write().unwrap();
                let agents = match &mut *agents {
                    Some(agents) => agents,
                    None => return Err(String::from("No agents were found\n")),
                };

                match agents.get_mut(command_vec[2]) {
                    Some(agent) => {
                        agent.put_forbid();
                        if *agent.get_status() == AgentStatus::Forbidden{
                            return Ok(String::from("\n"));
                        }
                        else {
                            return Err(String::from("Failed to move agent into forbidden\n"));
                        }
                    },
                    None => return Err(String::from("No agent were found\n")),
                }
            }
            else if command_vec[1] == "run" {
                let mut agents = AGENTS.write().unwrap();
                let agents = match &mut *agents {
                    Some(agents) => agents,
                    None => return Err(String::from("No agents were found\n")),
                };

                match agents.get_mut(command_vec[2]) {
                    Some(agent) => {
                        if *agent.get_status() != AgentStatus::Ready {
                            return Err(String::from("Agent is not ready to run\n"));
                        }
                        agent.put_running();

                        let msg = match agent.execute() {
                            Ok(_) => String::from("Agent run successfully\n"),
                            Err(e) => format!("Agent failed to run exit code is {:?}", e),
                        };

                        agent.update_last_run();
                        agent.put_ready();

                        return Ok(msg);
                    }
                    None => return Err(String::from("No agent were found\n")),
                }
            }
        }
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