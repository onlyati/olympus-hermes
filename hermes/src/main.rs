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

    let mut db = Database::new();
    db.create_table(String::from("Teszt1")).unwrap();
    
    match db.select_table_mut("Teszt1") {
        Some(table) => {
            for x in 0..100 {
                let upper_limit = x * 1_000;

                let now = Instant::now();
                for i in (0..upper_limit).rev() {
                    table.insert_or_update(format!("{}", i).as_str(), "Teszt value is here");
                    // table.insert_or_update("Update teszt", "Teszt value is here");
                }
                let elapsed = now.elapsed();
                println!("Elapsed time for {} inserts:\t {:.2?}", upper_limit, elapsed);

                table.remove(|_| {true});
            }
        },
        None => (),
    }
    
}
