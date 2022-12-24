#![allow(dead_code)]

use std::fmt;
use std::fmt::Display;
use std::io::{BufReader, Read, Write};
use std::process::{Command, Stdio};

use chrono::{Datelike, Timelike};

use crate::AGENTS;

//
// Enums
//

/// Enum to represent current status of agent
#[derive(PartialEq, Eq, Clone)]
pub enum AgentStatus {
    Ready,
    Running,
    Forbidden,
}

impl Display for AgentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = match self {
            AgentStatus::Forbidden => "Forbidden",
            AgentStatus::Ready => "Ready",
            AgentStatus::Running => "Running",
        };
        write!(f, "{}", display)
    }
}

/// Enum to represent agent message output type
#[derive(PartialEq, Eq)]
pub enum AgentOutputType {
    Info,
    Error,
}

impl Display for AgentOutputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = match self {
            AgentOutputType::Info => "I",
            AgentOutputType::Error => "E",
        };
        write!(f, "{}", display)
    }
}

//
// Structs
//

/// Struct for agent
#[derive(Clone)]
pub struct Agent {
    id: String,
    interval: u64,
    exe_path: String,
    log_path: String,
    conf_path: Vec<String>,
    status: AgentStatus,
    last_run: Option<String>,
}

impl Agent {
    /// Create new Agent
    pub fn new(id: String, interval: u64, exe_path: String, log_path: String , conf_path: Vec<String>) -> Self {
        Agent {
            id: id,
            interval: interval,
            exe_path: exe_path,
            conf_path: conf_path,
            log_path: log_path,
            status: AgentStatus::Ready,
            last_run: None,
        }
    }

    // Get agent status
    pub fn get_status(&self) -> &AgentStatus {
        return &self.status;
    }

    // Get the agent id
    pub fn get_id(&self) -> &str {
        return &self.id[..];
    }

    // Get interval of agent
    pub fn get_interval(&self) -> std::time::Duration {
        return std::time::Duration::new(self.interval, 0);
    }

    // Get when it was run last time
    pub fn get_last_run(&self) -> Option<&str> {
        match &self.last_run {
            Some(lr) => return Some(&lr[..]),
            None => None,
        }
    }

    // Update when agent was run last time
    pub fn update_last_run(&mut self) {
        let now = chrono::Local::now();
        let now = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());

        self.last_run = Some(now);
    }

    // Forbid agent to run
    pub fn put_forbid(&mut self) {
        self.status = AgentStatus::Forbidden;
    }

    pub fn put_ready(&mut self) {
        self.status = AgentStatus::Ready;
    }

    pub fn execute(&self) -> Result<(), Option<i32>> {
        let mut child = Command::new(&self.exe_path)
            .args(&self.conf_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        
        let mut stdout: Vec<AgentOutput> = Vec::new();
        let mut stderr: Vec<AgentOutput> = Vec::new();

        std::thread::scope(|spawner| {
            spawner.spawn(|| {
                let pipe = child.stdout.as_mut().unwrap();
                stdout = read_buffer(&mut BufReader::new(pipe));
            });
            spawner.spawn(|| {
                let pipe = child.stderr.as_mut().unwrap();
                stderr = read_buffer(&mut BufReader::new(pipe));
            });
        });

        let status = child.wait().unwrap();

        stdout.append(&mut stderr);
        stdout.sort_by(|a, b| a.time.cmp(&b.time));

        let mut log_file = match std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&self.log_path) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Failed to write log for {} agent due to: {}", self.id, e);
                    return Err(Some(-999));
                }
        };

        for msg in stdout {
            match write!(&mut log_file, "{} {} {}", msg.time, msg.out_type, msg.text) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Failed to wrrite log for {} agent due to: {}", self.id, e);
                    return Err(Some(-998));
                }
            };
        }

        if !status.success() {
            return Err(status.code());
        }
        
        return Ok(());
    }
}

/// Struct for agent output message
struct AgentOutput {
    time: String,
    text: String,
    out_type: AgentOutputType,
}

//
// Other functions
//

// Internal function, it is used to read the stdout and stderr of agent
fn read_buffer<T: Read>(reader: &mut BufReader<T>) -> Vec<AgentOutput> {
    const BUFFER_SIZE: usize = 128;
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut line = String::new();
    let mut messages: Vec<AgentOutput> = Vec::new();

    while let Ok(size) = reader.read(&mut buffer) {
        if size == 0 {
            break;
        }

        line += String::from_utf8_lossy(&buffer[0..size]).as_ref();

        for c in buffer {
            if c == b'\n' {
                let now = chrono::Local::now();
                let now = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());

                let lines: Vec<&str> = line.split("\n").collect();
                if lines.len() == 1 {
                    messages.push(AgentOutput { 
                        time: now.clone(), 
                        text: line + "\n", 
                        out_type: AgentOutputType::Info,
                    });
                    line = String::new();
                }
                else {
                    for i in 0..lines.len() - 1 {
                        let line = String::from(lines[i]);
                        messages.push(AgentOutput { 
                            time: now.clone(), 
                            text: line + "\n", 
                            out_type: AgentOutputType::Info,
                        });
                    }
                    line = String::from(lines[lines.len() - 1]);
                }

                
                continue;
            }
        }
        buffer = [0; BUFFER_SIZE];
    }

    return messages;
}

// This function can be called from a thread and it handle the agent running
pub async fn setup_agent(id: String) {
    loop {
        println!("Agent {} is starting...", id);

        let status = {
            let status: AgentStatus;
            let mut agents = AGENTS.write().unwrap();
            let agents = match &mut *agents {
                Some(agents) => agents,
                None => {
                    eprintln!("No agent config alive");
                    break;
                }
            };

            match agents.get_mut(&id) {
                Some(agent) => {
                    if agent.status == AgentStatus::Ready {
                        agent.status = AgentStatus::Running;
                    }
                    status = agent.status.clone();
                }
                None => {
                    eprintln!("Specified agent {} does not exist", id);
                    break;
                }
            };

            status
        };

        if status == AgentStatus::Running {
            let agents = AGENTS.read().unwrap();
            let agents = match &*agents {
                Some(agents) => agents,
                None => {
                    eprintln!("No agent config alive");
                    break;
                }
            };

            match agents.get(&id) {
                Some(agent) => {
                    match agent.execute() {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("Failed to execute agent {}, exit code {:?}", id, e);
                        }
                    };
                }
                None => {
                    eprintln!("Specified agent {} does not exist", id);
                    break;
                }
            }
        }
        else {
            println!("Agent {} cannot run but {}", id, status);
        }

        let interval = {
            let mut agents = AGENTS.write().unwrap();
            let agents = match &mut *agents {
                Some(agents) => agents,
                None => {
                    eprintln!("No agent config alive");
                    break;
                }
            };

            match agents.get_mut(&id) {
                Some(agent) => {
                    agent.update_last_run();

                    if agent.status != AgentStatus::Forbidden {
                        agent.status = AgentStatus::Ready;
                    }
                    agent.interval
                },
                None => {
                    eprintln!("Specified agent {} does not exist", id);
                    break;
                },
            }
        };
        
        println!("Agent {} is ended", id);
        tokio::time::sleep(tokio::time::Duration::new(interval, 0)).await;
    }
}