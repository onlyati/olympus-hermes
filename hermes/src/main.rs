use std::env;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::process::Command;

mod hermes_config;
use hermes_config::HermesConfig;

mod thread_pool;
use thread_pool::ThreadPool;

mod data_handler;
use data_handler::Group;
use data_handler::Item;


fn main() 
{
    // Read the arguments and parse it onto a structure
    let args: Vec<String> = env::args().collect();
    let config = parse_args(args);
    println!("{}", config);

    // Initialize ThreadPool
    let pool = ThreadPool::new(&config.threads);

    // Initailize and start TCP listening
    let listener = TcpListener::bind(&config.addr).expect(format!("Bind has failed to {}", &config.addr).as_str());

    for stream in listener.incoming()
    {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_request(stream);
        })
    }

    // End of Hermes
    println!("Hermes is shutting down...");
}

fn handle_request(mut stream: TcpStream)
{
    let mut buffer = [0; 4096];
    stream.read(&mut buffer).unwrap();
    //println!("Incoming data is: {}", String::from_utf8_lossy(&buffer[..]));

    let response = "I got it boss!";
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

/// Parse arguments
/// 
/// This function set default values for HermerConfig struct and read its parameters from argument.
/// 
/// # Input(s)
/// 
/// Input parameters, stored in Vec<String>.
/// 
/// # Return value
/// 
/// It builds a HermesConfig struct.
/// 
/// # Panics
/// 
/// - If specified parameter for --thread or -t is not a number
fn parse_args(args: Vec<String>) -> HermesConfig 
{
    let mut data_path: String = String::from("/usr/lib/onlyati/hermes/data");
    let mut addr: String = String::from("127.0.0.1:3030");
    let mut threads: usize = number_of_cpu_threads();

    for i in 0..args.len() 
    {
        match args[i].as_str()
        {
            "--data" | "-d"  => data_path = if i < args.len() - 1 { args[i + 1].clone() } else { data_path },
            "--port" | "-p"  => addr = if i < args.len() - 1 { args[i + 1].clone() } else { addr },
            "--threads" | "-t" => threads = if i < args.len() - 1 { args[i + 1].parse::<usize>().expect("Thread number is not a number") } else { threads },
            _ => (),
        }
    }

    return HermesConfig::new(data_path, addr, threads);
}

/// Count CPU threads
/// 
/// This method find out how many CPU threads are running based on `/usr/bin/grep -c ^processor /proc/info` output.
/// 
/// # Input(s)
/// 
/// No input.
/// 
/// # Return value
/// 
/// Number of CPU threads.
/// 
/// # Panics
/// 
/// - If it could not execute grep command or
/// - Command output parse onto `usize` has failed
fn number_of_cpu_threads() -> usize {
    let raw_output = Command::new("/usr/bin/grep")
        .arg("-c")
        .arg("^processor")
        .arg("/proc/cpuinfo")
        .output()
        .expect("Could not find out how many CPU thread has");

    let mut output = String::from(String::from_utf8_lossy(&raw_output.stdout));
    if output.ends_with('\n')
    {
        output.pop();
    }
    let count: usize = output.parse::<usize>().expect(format!("Could not parse CPU thread count onto number: [{}]", output).as_str());
    count
}
