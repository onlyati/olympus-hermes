use std::env;
use std::process::exit;
use std::net::TcpStream;
use std::io::{Write, Read};

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        println!("Missing input parameters!");
        exit(1);
    }

    args.remove(0);

    let mut stream = TcpStream::connect("127.0.0.1:3030").unwrap();

    let full_cmd = args.join(" ");
    let message = format!("{} {}", full_cmd.len(), full_cmd);

    let now = std::time::Instant::now();
    stream.write(message.as_bytes()).unwrap();
    
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    let elapsed = now.elapsed();
    println!("{response}");
    println!("Elapsed time: {:?}", elapsed);
}
