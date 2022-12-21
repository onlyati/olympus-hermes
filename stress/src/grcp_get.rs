use hermes::hermes_client::{HermesClient};
use hermes::{Key};

mod hermes {
    tonic::include_proto!("hermes");
}

#[tokio::main]
async fn main() {
    let mut joins: Vec<tokio::task::JoinHandle<()>> = Vec::new();

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

        let t1 = tokio::spawn(async move {
            let mut times: Vec<u128> = Vec::with_capacity(500_000 * std::mem::size_of::<u128>());
            let whole_now = std::time::Instant::now();
            let mut client = HermesClient::connect("http://0.0.0.0:9099").await.unwrap();

            for i in 0..500_000 {
                let pair = Key {
                    key: String::from("mem_limit"),
                    table: String::from("Monitoring"),
                };
                let request = tonic::Request::new(pair);

                let now = std::time::Instant::now();

                let _ = client.get(request).await;
                // println!("{:?}", response);
    
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
        let _ = tokio::join!(join);
    }
    
}
