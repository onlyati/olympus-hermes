#![allow(dead_code)]

use std::fmt;
use std::fmt::Display;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use chrono::{Datelike, Timelike};

//
// Enums
//

/// Enum to represent current status of agent
#[derive(PartialEq, Eq)]
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
pub struct Agent {
    id: String,
    interval: u32,
    exe_path: String,
    log_path: Box<Path>,
    conf_path: Vec<String>,
    status: AgentStatus,
}

impl Agent {
    /// Create new Agent
    pub fn new(id: String, interval: u32, exe_path: String, log_path: Box<Path> , conf_path: Vec<String>) -> Self {
        Agent {
            id: id,
            interval: interval,
            exe_path: exe_path,
            conf_path: conf_path,
            log_path: log_path,
            status: AgentStatus::Ready,
        }
    }

    // Get agent status
    pub fn get_status(&self) -> &AgentStatus {
        return &self.status;
    }

    // Forbid agent to run
    pub fn put_forbid(&mut self) {
        self.status = AgentStatus::Forbidden;
    }

    pub fn put_ready(&mut self) {
        self.status = AgentStatus::Ready;
    }

    pub fn execute(&mut self) -> Result<(), Option<i32>> {
        if self.status != AgentStatus::Ready {
            return Ok(());
        }

        self.status = AgentStatus::Running;

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
        self.status = AgentStatus::Ready;

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
            match writeln!(&mut log_file, "{} {} {}", msg.time, msg.out_type, msg.text) {
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
                messages.push(AgentOutput { 
                    time: now, 
                    text: line, 
                    out_type: AgentOutputType::Info,
                });
                line = String::new();
                continue;
            }
        }
        buffer = [0; BUFFER_SIZE];
    }

    return messages;
}