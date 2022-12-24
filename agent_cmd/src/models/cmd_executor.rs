use std::fmt;
use std::fmt::Display;
use std::io::{BufReader, Read, BufRead};
use std::process::{Command, Stdio};

use chrono::{Datelike, Timelike};

//
// Enums
//

/// Enum to represent agent message output type
#[derive(PartialEq, Eq, Copy, Clone)]
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

/// Struct for agent output message
#[derive(Clone, Eq, PartialEq)]
pub struct AgentOutput {
    pub time: String,
    pub text: String,
    pub out_type: AgentOutputType,
}

pub struct Error {
    pub output: Vec<AgentOutput>,
    pub exit_code: Option<i32>,
}

//
// Functions
//

pub fn execute(cmd: String, args: Vec<String>) -> Result<Vec<AgentOutput>, Error> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

        let mut stdout: Vec<AgentOutput> = Vec::new();
        let mut stderr: Vec<AgentOutput> = Vec::new();

        std::thread::scope(|spawner| {
            spawner.spawn(|| {
                let pipe = child.stdout.as_mut().unwrap();
                stdout = read_buffer(&mut BufReader::new(pipe), AgentOutputType::Info);
            });
            spawner.spawn(|| {
                let pipe = child.stderr.as_mut().unwrap();
                stderr = read_buffer(&mut BufReader::new(pipe), AgentOutputType::Error);
            });
        });

        let status = child.wait().unwrap();

        stdout.append(&mut stderr);
        stdout.sort_by(|a, b| a.time.cmp(&b.time));

        if !status.success() {
            return Err(Error {
                output: stdout,
                exit_code: status.code(),
            });
        }

        return Ok(stdout);
}

// Internal function, it is used to read the stdout and stderr of agent
fn read_buffer<T: Read>(reader: &mut BufReader<T>, out_type: AgentOutputType) -> Vec<AgentOutput> {
    let mut line = String::new();
    let mut messages: Vec<AgentOutput> = Vec::new();

    while let Ok(size) = reader.read_line(&mut line) {
        if size == 0 {
            break;
        }

        let now = chrono::Local::now();
        let now = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());
        messages.push(AgentOutput { 
            time: now, 
            text: line, 
            out_type: out_type 
        });

        line = String::new();
    }

    return messages;
}