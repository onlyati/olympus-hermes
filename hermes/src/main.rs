use std::env;
use std::collections::HashMap;

mod services;
use services::data::Database;
use services::process::Pool;

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

    let pool = Pool::new(4).unwrap();

    pool.execute(String::from("Hello")).unwrap();
    pool.execute(String::from("Hogy vagy?")).unwrap();
    pool.execute(String::from("ASDasdASD")).unwrap();
    pool.execute(String::from("Huehuehue")).unwrap();
    pool.execute(String::from("Cecília")).unwrap();
    
}
