use std::env;
use std::collections::HashMap;

mod services;
use services::data::Database;
use services::process::Pool;

use std::time::Instant;

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

    println!("-----------------------------");

    let mut db = Database::new();
    db.create_table(String::from("Teszt1")).unwrap();
    db.create_table(String::from("Error")).unwrap();
    db.create_table(String::from("Attila")).unwrap();
    db.create_table(String::from("Teszt2")).unwrap();
    db.create_table(String::from("Teszt4")).unwrap();
    db.create_table(String::from("Teszt3")).unwrap();
    db.create_table(String::from("Batch")).unwrap();

    for table in db.get_tables() {
        println!("{}", table.get_name());
    }

    println!("-----------------------------");
    db.drop_table("Teszt2").unwrap();

    for table in db.get_tables() {
        println!("{}", table.get_name());
    }

    println!("-----------------------------");
    for table in db.filter_tables(|key| { key.starts_with("Teszt")}) {
        println!("{}", table.get_name());
    }

    println!("-----------------------------");

    match db.select_table_mut("Teszt1") {
        Some(table) => {
            table.insert_or_update("Hello", "value");
            table.insert_or_update("Xerxész", "value");
            table.insert_or_update("123abc", "value");
            table.insert_or_update("Cecília", "value");
            table.insert_or_update("Béla", "value");
            table.insert_or_update("Balta", "value");
            table.insert_or_update("Kelemen", "value");
            table.insert_or_update("Batch-Error-1", "value");
            table.insert_or_update("Batch-Error-2", "value");
            table.insert_or_update("Batch-Error-3", "value");
            table.insert_or_update("Batch-Error-4", "value");
            table.insert_or_update("Batch-Status-1", "value");

            let list = table.key_start_with("Batch-Error");
            println!("{:?}", list);

            println!("{}", table);
        },
        None => (),
    }

    println!("-----------------------------");

    match db.select_table_mut("Teszt1") {
        Some(table) => {
            println!("Count\t\tInsert\t\tGet\t\tRemove\t\tFilter");
            for x in 5..51 {
                let upper_limit = x * 1_000;

                let now = Instant::now();
                for i in (0..upper_limit).rev() {
                    table.insert_or_update(format!("{}", i).as_str(), "Teszt value is here");
                    // table.insert_or_update("Update teszt", "Teszt value is here");
                }
                let elapsed_insert = now.elapsed();

                let now = Instant::now();
                for i in (0..upper_limit).rev() {
                    let _ = table.get_value(format!("{}", i).as_str());
                }
                let elapsed_get = now.elapsed();

                let now = Instant::now();
                for _ in 0..upper_limit {
                    let _ = table.key_start_with("2000");
                }
                let elapsed_filter = now.elapsed();

                let now = Instant::now();
                table.remove(|_| {true});
                let elapsed_remove = now.elapsed();

                println!("{}\t\t{:.2?}\t\t{:.2?}\t\t{:.2?}\t\t{:.2?}", upper_limit, elapsed_insert, elapsed_get, elapsed_remove, elapsed_filter);
            }
        },
        None => (),
    }
    
}
