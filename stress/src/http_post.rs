use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let mut joins: Vec<tokio::task::JoinHandle<()>> = Vec::new();

    for j in 0..1 {

        let t1 = tokio::spawn(async move {
            let mut times: Vec<u128> = Vec::with_capacity(500_000 * std::mem::size_of::<u128>());
            let whole_now = std::time::Instant::now();

            let client = reqwest::Client::new();


            for i in 0..500_000 {
                
                let mut body: HashMap<String, String> = HashMap::new();
                body.insert("key".to_string(), format!("/root/a{}", i));
                body.insert("value".to_string(), format!("a{}", i));

                let now = std::time::Instant::now();

                client.post("http://127.0.0.1:3032/db").json(&body).send().await.expect("Failed to send request");
    
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
    
            println!("Full time: {:?}\nAverage time for thread #{}: {} us", total, j, avg);
        });
        
        joins.push(t1);
    }

    for join in joins {
        let _ = tokio::join!(join);
    }
}