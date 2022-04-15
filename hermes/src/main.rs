use std::env;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::process::Command;
use std::collections::HashMap;

use onlyati_http::parser::HttpResponse;
use onlyati_http::parser::EndPointType;
use onlyati_http::parser::RequestInfo;
use onlyati_http::parser::RequestResponse;
use onlyati_http::endpoints::EndPointCollection;
use onlyati_http::endpoints::EndPointAction;

use once_cell::sync::OnceCell;

use std::sync::Arc;
use std::sync::Mutex;

mod thread_pool;
use thread_pool::ThreadPool;

mod data_handler;
use data_handler::Group;

static DATA: OnceCell<Mutex<Group>> = OnceCell::new();

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
    let mut config: HashMap<String, String> = HashMap::new();

    match config_tmp {
        Ok(r) => config = r,
        Err(e) => {
            println!("Error during config reading: {}", e);
            return;
        },
    }

    // Initialize data structure to store it
    let mut data_group = Group::new();
    let mut_data = DATA.set(Mutex::new(data_group));
    if let Err(_) = mut_data {
        println!("Error during mutex data bind!");
        return;
    }

    // Setup REST API endpoints
    let mut endpoints = EndPointCollection::new();
    endpoints.add("/get", EndPointType::GET, get_value);
    endpoints.add("/set", EndPointType::POST, set_value);

    // Setup Threadpool
    if !config.contains_key("threads") {
        config.insert(String::from("threads"), number_of_cpu_threads().to_string());
    }
    
    println!("Write out the config:");
    for (key, value) in &config {
        println!("{} -> {}", key, value);
    }

    let pool: ThreadPool;
    match &config.get("threads") {
        Some(v) => {
            let count: usize = v.parse::<usize>().unwrap();
            pool = ThreadPool::new(&count);
        },
        None => {
            println!("Thread number is not specified!");
            return;
        },
    }

    // Initailize and start TCP listening
    let listener: TcpListener;
    match &config.get("address") {
        Some(v) => listener = TcpListener::bind(v).expect(format!("Bind has failed to {}", v).as_str()),
        None => {
            println!("Address is not specified");
            return;
        }
    }

    let endp = Arc::new(Mutex::new(endpoints));

    // Start listening
    println!("Start listening...");
    for stream in listener.incoming()
    {
        let endp_arc = Arc::clone(&endp);
        let stream = stream.unwrap();
        pool.execute(move || {
            handle_request(stream, endp_arc);
        });
    }

    // End of Hermes
    println!("Hermes is shutting down...");
}

fn handle_request(mut stream: TcpStream, endpoints: Arc<Mutex<EndPointCollection>>) {
    let mut buffer = [0; 4096];
    buffer.fill(0x00);
    stream.read(&mut buffer).unwrap();
    
    // Default answer
    let mut response = RequestResponse::new(HttpResponse::BadRequest, HashMap::new(), String::from(""));

    // Parse the incoming request onto a structure
    let input = String::from_utf8_lossy(&buffer[..]).trim().to_string();
    
    let mut index: usize = 0;
    match input.find('\0') {
        Some(r) => index = r,
        _ => (),
    }

    let input = String::from(&input[0..index]);

    let infos = RequestInfo::new(&input[..]);
    if let Some(info) = infos {
        // If parse was successful, then find endpoint for it
        {
            let endp = endpoints.lock().unwrap();
            response = endp.execute(info);
        }
    }
 
    // Create a text HTTP response from structure
    let final_response = response.print();

    stream.write(final_response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn set_value(info: &RequestInfo) -> RequestResponse {
    let mut header: HashMap<String, String> = HashMap::new();
    header.insert(String::from("Content-Type"), String::from("plain/text"));

    let name: String;
    match info.parameters.get("name") {
        Some(r) => name = String::from(r),
        None => return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missign parameter: name")),
    }

    let value: String;
    if !info.body.trim().is_empty() {
        value = String::from(info.body.trim());
    }
    else {
        return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missing in body: value of key"));
    }

    let data_mut = DATA.get();
    match data_mut {
        Some(r) => {
            {
                let mut data = data_mut.unwrap().lock().unwrap();
                data.insert_or_update(&name[..], &value[..]);
            }
            return RequestResponse::new(HttpResponse::Ok, header, String::from(""));
        },
        None => return RequestResponse::new(HttpResponse::InternalServerError, header, String::from("Sorry :-(")),
    }

    return RequestResponse::new(HttpResponse::Ok, HashMap::new(), String::from(""));
}

fn get_value(info: &RequestInfo) -> RequestResponse {
    let mut header: HashMap<String, String> = HashMap::new();
    header.insert(String::from("Content-Type"), String::from("plain/text"));

    let name: String;
    match info.parameters.get("name") {
        Some(r) => name = String::from(r),
        None => return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missign parameter: name")),
    }

    let data_mut = DATA.get();
    match data_mut {
        Some(r) => {
            {
                let mut data = data_mut.unwrap().lock().unwrap();
                match data.find(&name[..]) {
                    Some(r) => {
                        let resp: String = format!("{}\n{}\n{}\n", r.0, r.2, r.1);
                        return RequestResponse::new(HttpResponse::Ok, header, resp);
                    },
                    None => return RequestResponse::new(HttpResponse::NotFound, header, String::from("Key was not found")),
                }
            }
        },
        None => return RequestResponse::new(HttpResponse::InternalServerError, header, String::from("Sorry :-(")),
    }
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
