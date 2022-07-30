#![allow(dead_code)]
use std::fmt;
use std::mem::size_of;
use std::thread;
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;

pub enum Order{
    Execute(String),
    Terminate,
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Order::Terminate => return write!(f, "Termination"),
            Order::Execute(e) => return write!(f, "Order: {}", e),
        }
    }
}

pub struct Pool {
    distributor: Sender<Order>,
    threads: Vec<Worker>,
}

impl Pool {
    pub fn new(cores: usize) -> Result<Pool, String> {
        println!("Creating the pool...");
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut threads: Vec<Worker> = Vec::with_capacity(cores * size_of::<Worker>());

        for i in 0..cores {
            threads.push(Worker::new(i, receiver.clone()));
        }

        println!("Pool is created");

        return Ok(Pool {
            distributor: sender,
            threads: threads,
        });
    }

    pub fn execute(&self, order: String) -> Result<(), String> {
        match self.distributor.send(Order::Execute(order)) {
            Ok(_) => return Ok(()),
            Err(e) => return Err(format!("{:?}", e)),
        }
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        println!("Stopping the pool...");

        for _ in &self.threads {
            self.distributor.send(Order::Terminate).unwrap();
        }

        for worker in &mut self.threads {
            let thread = worker.thread.take().unwrap();
            thread.join().unwrap();
        }

        println!("Pool has stopped");
    }
}

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Order>>>) -> Worker {
        
        let (send, rec) = mpsc::channel();

        let thread = thread::spawn(move || {
            println!("Thread #{} has started", id);
            send.send(true).unwrap();
            loop {
                let todo: Order = { 
                    let rec = match receiver.lock() {
                        Ok(m) => m,
                        Err(e) => {
                            println!("{:?}", e);
                            return;
                        }
                    };

                    let t = match rec.recv() {
                        Ok(s) => s,
                        Err(e) => {
                            println!("{:?}", e);
                            return;
                        }
                    };

                    t
                };

                match todo {
                    Order::Terminate => {
                        println!("#{} has to terminate", id);
                        break;
                    },
                    Order::Execute(s) => {
                        println!("#{} has to do this: {}", id, s);
                        std::thread::sleep(std::time::Duration::new(0, 750));
                    }
                }
            }
        });

        match rec.recv() {
            Ok(_) => {
                return Worker { 
                    id: id,
                    thread: Some(thread),
                };
            }
            Err(e) => {
                println!("Thread creation failed for #{}: {:?}", id, e);
                return Worker { 
                    id: id,
                    thread: None,
                };
            }
        }
    }
}

impl fmt::Display for Worker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.thread {
            Some(_) => return write!(f, "ID: {}, Status: running", self.id),
            None => return write!(f, "ID: {}, Status: Stopped", self.id),
        }
        
    }
}