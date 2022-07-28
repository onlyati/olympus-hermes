use std::env;
use std::net::TcpListener;
use std::process::Command;
use std::collections::HashMap;

mod thread_pool;

use thread_pool::ThreadPool;

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

    

    // End of Hermes
    println!("Hermes is shutting down...");
}
