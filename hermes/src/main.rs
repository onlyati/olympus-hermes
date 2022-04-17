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

use once_cell::sync::OnceCell;

use std::sync::Arc;
use std::sync::Mutex;

mod thread_pool;
use thread_pool::ThreadPool;

mod data_handler;
use data_handler::Group;

static DATA: OnceCell<Mutex<Group>> = OnceCell::new();
const BUFFER_SIZE: usize = 4096;

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
    endpoints.add("/item/get", EndPointType::GET, get_value);
    endpoints.add("/item/set", EndPointType::POST, set_value);
    endpoints.add("/item/remove", EndPointType::DELETE, remove_value);
    endpoints.add("/item/filter", EndPointType::GET, filter_value);

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

/// Handle requeste
/// 
/// Function which is passed to each thread to execute:
/// 1. Read the incomcing data
/// 2. Parse it onto `RequestResponse` request
/// 3. Calling execution for endpoints
/// 4. Send the request back to the caller
fn handle_request(mut stream: TcpStream, endpoints: Arc<Mutex<EndPointCollection>>) {
    let mut incoming_data: String = String::new();
    let mut buffer_count: usize = BUFFER_SIZE;
    
    while buffer_count == BUFFER_SIZE {
        let mut buffer = [0; BUFFER_SIZE];
        buffer.fill(0x00);
        match stream.read(&mut buffer) {
            Ok(r) => {
                incoming_data = incoming_data + String::from_utf8_lossy(&buffer[0..r]).trim();
                buffer_count = r;
            },
            Err(_) => {
                let mut header: HashMap<String, String> = HashMap::new();
                header.insert(String::from("Content-Type"), String::from("plain/text"));
                let response = RequestResponse::new(HttpResponse::InternalServerError, header, String::from("Sorry :-("));
                stream.write(response.print().as_bytes()).unwrap();
                stream.flush().unwrap();
            },
        }
    }
    
    // Default answer
    let mut response = RequestResponse::new(HttpResponse::BadRequest, HashMap::new(), String::from(""));

    let infos = RequestInfo::new(&incoming_data[..]);
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

fn filter_value(info: &RequestInfo) -> RequestResponse {
    // Response will be plain text
    let mut header: HashMap<String, String> = HashMap::new();
    header.insert(String::from("Content-Type"), String::from("plain/text"));

    // Get the filter value
    let filter: String;
    match info.parameters.get("name") {
        Some(r) => filter = String::from(r),
        None => return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missing paramter: name")),
    }

    let data_mut = DATA.get();
    match data_mut {
        Some(_) => {
            let mut list: Vec<String>;
            {
                let mut data = data_mut.unwrap().lock().unwrap();
                match data.filter(&filter[..]) {
                    Some(v) => list = v,
                    None => list = Vec::new(),
                }
            }

            let mut answer: String = format!("{}\n", list.len());
            for key in list {
                answer = answer + &key[..] + "\n";
            }

            return RequestResponse::new(HttpResponse::Ok, header, answer);

        },
        None => return RequestResponse::new(HttpResponse::InternalServerError, header, String::from("Sorry :-(")),
    }
}

/// Delete value
/// 
/// This is called for DELETE /item/remove?name=xxxxxx request.
fn remove_value(info: &RequestInfo) -> RequestResponse {
    // Response will be plain text
    let mut header: HashMap<String, String> = HashMap::new();
    header.insert(String::from("Content-Type"), String::from("plain/text"));

    // Save the name of the key
    let name: String;
    match info.parameters.get("name") {
        Some(r) => name = String::from(r),
        None => return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missing parameter: name")),
    }

    let data_mut = DATA.get();
    match data_mut {
        Some(_) => {
            let mut answer: String;
            {
                let mut data = data_mut.unwrap().lock().unwrap();
                match data.delete(&name[..]) {
                    Some(v) => answer = v,
                    None => answer = String::from("Key was not exist"),
                }
                return RequestResponse::new(HttpResponse::Ok, header, answer);
            }
        },
        None => return RequestResponse::new(HttpResponse::InternalServerError, header, String::from("Sorry :-(")),
    }
}

/// Set value
/// 
/// This is called for POST /item/set?name=xxxxx request. Value of the key is in the `info.body`
fn set_value(info: &RequestInfo) -> RequestResponse {
    // Response will be plain text
    let mut header: HashMap<String, String> = HashMap::new();
    header.insert(String::from("Content-Type"), String::from("plain/text"));

    // Save the name of the key
    let name: String;
    match info.parameters.get("name") {
        Some(r) => name = String::from(r),
        None => return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missing parameter: name")),
    }

    // Save the value from the body
    let value: String;
    if !info.body.trim().is_empty() {
        value = String::from(info.body.trim());
    }
    else {
        return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missing in body: value of key"));
    }

    // Try to insert incoming data and assemble the response accordingly
    let data_mut = DATA.get();
    match data_mut {
        Some(_) => {
            let mut answer: String = String::new();
            {
                let mut data = data_mut.unwrap().lock().unwrap();
                match data.insert_or_update(&name[..], &value[..]) {
                    Some(v) => answer = v.clone(),
                    None => answer = String::from("Key already has this value, no update"),
                }
            }
            return RequestResponse::new(HttpResponse::Ok, header, answer);
        },
        None => return RequestResponse::new(HttpResponse::InternalServerError, header, String::from("Sorry :-(")),
    }
}

/// Get value
/// 
/// This is called for GET /item/get?name=xxxx request. It returns the value of the key.
fn get_value(info: &RequestInfo) -> RequestResponse {
    // Response will be plain text
    let mut header: HashMap<String, String> = HashMap::new();
    header.insert(String::from("Content-Type"), String::from("plain/text"));

    // Get key name from parameters
    let name: String;
    match info.parameters.get("name") {
        Some(r) => name = String::from(r),
        None => return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missing parameter: name")),
    }

    // Try to find data, set response accordingly
    let data_mut = DATA.get();
    match data_mut {
        Some(r) => {
            {
                let mut data = data_mut.unwrap().lock().unwrap();
                match data.find(&name[..]) {
                    Some(r) => {
                        let resp: String = format!("{}\n{}\n{}\n", r.1, r.0, r.2);
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
