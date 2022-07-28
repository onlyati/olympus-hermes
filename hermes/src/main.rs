use std::env;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod services;
use services::data::Database;

fn main() 
{
    // Read the arguments and parse it onto a structure
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Config path must be specified as parameter!");
        return;
    }

    // Read configuration from file
    let config_tmp = onlyati_config::read_config(args[1].as_str());
    let config: HashMap<String, String>;

    match config_tmp {
        Ok(r) => config = r,
        Err(e) => {
            println!("Error during config reading: {}", e);
            return;
        },
    }

    println!("Settings:");
    for setting in &config {
        println!("{} -> {}", setting.0, setting.1);
    }

    let mut db = Database::new();
    let _ = db.create_table(String::from("Xerxes"));
    let _ = db.create_table(String::from("Xerxes"));
    let _ = db.create_table(String::from("Elemér"));
    let _ = db.create_table(String::from("Elemér"));
    let _ = db.create_table(String::from("Dénes"));
    let _ = db.create_table(String::from("Cecil"));
    let _ = db.create_table(String::from("Béla"));
    let _ = db.create_table(String::from("Badacsony"));

    match db.select_table_mut("Elemér") {
        Some(table) => {
            table.insert_or_update("Teszt1", "Én vagyok az érték");
            table.insert_or_update("Mambo No 5", "I will survive");
            table.insert_or_update("Teszt2", "Én is érték vagyok");
            table.insert_or_update("Teszt3", "Én is, én is!");
        },
        None => (),
    }

    match db.select_table_mut("Cecil") {
        Some(table) => {
            table.insert_or_update("Teszt1", "Hehe, én Cecilben vagyok");
        },
        None => (),
    }

    match db.select_table_mut("Elemér") {
        Some(table) => {
            table.insert_or_update("Teszt2", "Én most úgy felülírlak");
        },
        None => (),
    }

    for table in db.get_tables() {
        match db.select_table(table.get_name()) {
            Some(table) => {
                let records = table.filter(|_| {true});
                for record in records {
                    println!("{} => {}", table.get_name(), record);
                }
            },
            None => (),
        }
    }

    println!("-------------------------------------------------");

    match db.select_table_mut("Elemér") {
        Some(table) => {
            match table.remove(|key| {key.starts_with("Teszt")}) {
                Some(count) => println!("{} element is deleted", count),
                None => println!("0 element is deleted"),
            }
        },
        None => (),
    }

    for table in db.get_tables() {
        match db.select_table(table.get_name()) {
            Some(table) => {
                let records = table.filter(|_| {true});
                for record in records {
                    println!("{} => {}", table.get_name(), record);
                }
            },
            None => (),
        }
    }
    
}
