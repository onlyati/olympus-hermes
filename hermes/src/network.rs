use std::mem::size_of;
use std::time::Duration;
use std::io::{Read, Write, BufReader};
use std::net::TcpStream;
use std::sync::{Arc, RwLock};

use crate::services::data::Database;
use crate::services::parser;

pub fn handle_connection(mut stream: TcpStream, db: Arc<RwLock<Database>>) {
    let now = std::time::Instant::now();

    let buffer = BufReader::new(&stream);

    let mut length_u8: Vec<u8> = Vec::with_capacity(5 * size_of::<usize>());   // Store bytes while readin, itis the message length
    let mut length: usize = 0;                                                 // This will be the parsed lenght from length_u8

    let mut msg_u8: Vec<u8> = Vec::new();                                      // Store message bytes

    let mut index = 0;                                                  // Index and read_msg are some variable for parsing incoming message
    let mut read_msg: bool = false;

    /*-------------------------------------------------------------------------------------------*/
    /* Read message from the buffer and parse it accordingly                                     */
    /*-------------------------------------------------------------------------------------------*/
    for byte in buffer.bytes() {
        match byte {
            Ok(b) => {
                /* It was the first space, first word must be a number which is the length of the subsequent message */
                if b == b' ' && !read_msg {
                    let msg_len_t = String::from_utf8(length_u8.clone()).unwrap();
                    length = match msg_len_t.parse::<usize>() {
                        Ok(v) => v,
                        Err(_) => {
                            let _ = stream.write_all(b"First word must be a number which is the lenght of message\n");
                            return;
                        }
                    };
                    msg_u8 = Vec::with_capacity(length);
                    read_msg = true;
                    continue;
                }

                // Set timeout to avoid infinite waiting on the stream
                stream.set_read_timeout(Some(Duration::new(0, 250))).unwrap();

                /* Read from buffer */
                if read_msg {
                    msg_u8.push(b);
                    index += 1;
                    if index == length {
                        break;
                    }
                    continue;
                }
                else {
                    length_u8.push(b);
                    continue;
                }
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                let _ = stream.write_all(b"ERROR: Request is not complete within time\n");
                return;
            },
            Err(e) => {
                println!("Unexpected error: {:?}", e);
                let _ = stream.write_all(b"ERROR: Internal server error during stream reading\n");
                return;
            },
        }
    }

    if !read_msg {
        /* This happen when the first world was not a number and new line was incoming */
        let _ = stream.write_all(b"First word must be a number which is the lenght of message\n");
        return;
    }

    let command = String::from_utf8(msg_u8).unwrap();
    
    let response: String = match parser::parse_db_command(&command[..], db) {
        Ok(s) => s,
        Err(s) => s,
    };
    
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

    let elapsed = now.elapsed();
    println!("Execution time: {:?}", elapsed);
}