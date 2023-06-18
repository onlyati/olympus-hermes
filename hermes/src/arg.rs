use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(author, about, long_about)]
pub struct ShellParms {
    /// Specify the action what to do
    #[command(subcommand)]
    pub action: Action,
}

#[derive(Parser, Debug, Clone)]
#[command(author, about, long_about)]
pub struct Parameters {
    /// Start Hermes CLI
    #[command(subcommand)]
    pub mode: Mode,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Mode {
    /// Use Hermes CLI mode        
    Cli(CliArgs),

    /// Start Hermes server
    Server {
        /// Parameters for server
        #[arg(long, short)]
        config: String,
    },

    /// Start a shell to issue CLI commands
    Shell {
        /// Where it should connect
        /// Allowed formats:
        /// - <protocol>://<hostname>:<port>, for example ws://127.0.0.1:3043
        /// - cfg://<definition-name>, for example: cfg://atihome, it will search  or hostname and CA certificate
        #[arg(short = 'H', long, verbatim_doc_comment, value_parser = check_hostname)]
        hostname: Option<String>,
    }
}

#[derive(Args, Debug, Clone)]
pub struct CliArgs {
    /// Specify the action what to do
    #[command(subcommand)]
    pub action: Action,

    /// Where it should connect
    /// Allowed formats:
    /// - <protocol>://<hostname>:<port>, for example ws://127.0.0.1:3043
    /// - cfg://<definition-name>, for example: cfg://atihome, it will search  or hostname and CA certificate
    #[arg(short = 'H', long, verbatim_doc_comment, value_parser = check_hostname)]
    pub hostname: String,

    /// Config file for connection details
    #[arg(short, long, default_value_t = String::from("/etc/olympus/hermes/client.toml"))]
    pub config: String,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Action {
    /// Get a value of a key
    Get {
        /// Specify the name of the key
        #[arg(long, short)]
        key: String,
    },

    /// Set value to a key
    Set {
        /// Specify the name of the key
        #[arg(long, short)]
        key: String,

        /// Specify the value for the key
        #[arg(long, short)]
        value: String,
    },

    /// Remove specified key
    RemKey {
        /// Specify the name of the key
        #[arg(long, short)]
        key: String,
    },

    /// Remove path
    RemPath {
        /// Specify the name of the key
        #[arg(long, short)]
        key: String,
    },

    /// List keys
    ListKeys {
        /// Specify the name of the key
        #[arg(long, short)]
        key: String,
    },

    /// Send trigger for hooks
    Trigger {
        /// Specify the name of the key
        #[arg(long, short)]
        key: String,

        /// Specify the value for the key
        #[arg(long, short)]
        value: String,
    },

    /// Create new hook
    SetHook {
        /// Key prefix for hook sending
        #[arg(long, short)]
        prefix: String,

        /// Address where the hook is sent
        #[arg(long, short)]
        link: String,
    },

    /// Check that a hook exists
    GetHook {
        /// Key prefix for hook sending
        #[arg(long, short)]
        prefix: String,
    },

    /// List hooks
    ListHooks {
        /// Key prefix for hook sending
        #[arg(long, short)]
        prefix: String,
    },

    /// Remove existing hook
    RemHook {
        #[arg(long, short)]
        /// Key prefix for hook sending
        prefix: String,

        /// Address where the hook is sent
        #[arg(long, short)]
        link: String,
    },

    /// Suspend file writing for database log
    SuspendLog,

    /// Resule file writing for database log
    ResumeLog,

    /// Execute lua script
    Exec {
        /// Specify the name of the key
        #[arg(long, short)]
        key: String,

        /// Specify the value for the key
        #[arg(long, short)]
        value: String,

        /// Specify name of the script
        #[arg(long, short)]
        script: String,

        /// Specify parameter that will be passed to script
        #[arg(long, short)]
        parms: Option<String>,

        /// Final value should be written or it is just a trigger event
        #[arg(long)]
        save: bool,
    },

    /// Push value to a queue
    Pop {
        /// Specify the name of the key
        #[arg(long, short)]
        key: String,
    },

    /// Pop value from a queue
    Push {
        /// Specify the name of the key
        #[arg(long, short)]
        key: String,

        /// Specify the value for the key
        #[arg(long, short)]
        value: String,
    },
}

fn check_hostname(s: &str) -> Result<String, String> {
    if !s.starts_with("cfg://") && !s.starts_with("ws://") {
        return Err(String::from(
            "Protocol for hostname can be http:// or https:// or cfg://. ",
        ));
    }

    if s.starts_with("http://") || s.starts_with("https://") {
        if !s.contains(':') {
            return Err(String::from(
                "Port number is not specified after the hostname. ",
            ));
        } else {
            let port = s.split(':').nth(2);
            match port {
                Some(p) => match p.parse::<u32>() {
                    Ok(num) => {
                        if num > 65535 {
                            return Err(String::from("Port number can be between 0..65535"));
                        }
                    }
                    Err(_) => {
                        return Err(String::from("Failed to convert port number to numbers"));
                    }
                },
                None => {
                    return Err(String::from(
                        "Port number is not specified after the hostname. ",
                    ))
                }
            }
        }
    }

    return Ok(String::from(s));
}
