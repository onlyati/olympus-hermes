use std::net::TcpStream;
use std::io::{Write, Read};
use std::thread::JoinHandle;

fn main() {
    let mut joins: Vec<JoinHandle<()>> = Vec::new();

    for j in 0..1 {
        let table = {
            if j % 4 == 0 {
                "Report"
            }
            else if j % 4 == 1 {
                "Error"
            }
            else if j % 4 == 2 {
                "Batch"
            }
            else {
                "Monitoring"
            }
        };

        let t1 = std::thread::spawn(move || {
            let mut times: Vec<u128> = Vec::with_capacity(500_000 * std::mem::size_of::<u128>());
            let whole_now = std::time::Instant::now();
            for i in 0..500_000 {
                let cmd = format!("set data('key{}', 'Hello ez itt a {}') in {};", i, i, table);
                let cmd = format!("{} {}", cmd.len(), cmd);
                let now = std::time::Instant::now();
    
                let mut stream = TcpStream::connect("127.0.0.1:3030").unwrap();
                stream.write(cmd.as_bytes()).unwrap();
                let mut result = String::new();
                stream.read_to_string(&mut result).unwrap();
    
                let elapsed = now.elapsed();
                times.push(elapsed.as_micros());
    
                if i % 100_000 == 0 {
                    println!("Thread #{} has send {} requests", j, i);
                }
            }
    
            let mut avg: u128 = 0;
            let mut total: u128 = 0;
            for time in &times {
                avg += time;
                total += time;
            }
    
            avg = avg / (times.len() as u128);
        
            let total = whole_now.elapsed();
    
            println!("Full time: {:?}\nAverage time for thread #{} with {}: {} us", total, j, table, avg);
        });
        
        joins.push(t1);
    }

    for join in joins {
        join.join().unwrap();
    }
    
}
