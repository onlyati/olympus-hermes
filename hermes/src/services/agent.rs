#![allow(dead_code)]

use std::io::{BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};

use chrono::{Datelike, Timelike};

pub enum AgentStatus {
    Ready,
    Running,
    Forbidden,
}

pub struct Agent {
    pub id: String,
    pub interval: u32,
    pub exe_path: String,
    pub log_path: Box<Path>,
    pub conf_path: Vec<String>,
    pub status: AgentStatus,
}

impl Agent {
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

    pub fn get_status(&self) -> &AgentStatus {
        return &self.status;
    }

    pub fn forbid(&mut self) {
        self.status = AgentStatus::Forbidden;
    }

    pub fn execute(&mut self) -> Result<(), Option<i32>> {
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

        stdout.append(&mut stderr);
        stdout.sort_by(|a, b| a.time.cmp(&b.time));

        let status = child.wait().unwrap();
        self.status = AgentStatus::Ready;

        if !status.success() {
            return Err(status.code());
        }
        
        return Ok(());
    }
}

pub enum AgentOutputType {
    Info,
    Error,
}

pub struct AgentOutput {
    time: String,
    text: String,
    out_type: AgentOutputType,
}

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
                let now = format!("{}-{}-{} {}:{}:{}", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());
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