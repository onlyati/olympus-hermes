use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(author, about, long_about)]
pub struct Args {
    /// Specify the action what to do
    #[command(subcommand)]
    pub action: Action,

    /// Where it should connect
    /// Allowed formats:
    /// - <protocol>://<hostname>:<port>, for example http://127.0.0.1:3041
    /// - cfg://<definition-name>, for example: cfg://atihome, it will search  or hostname and CA certificate
    #[arg(short = 'H', long, verbatim_doc_comment, value_parser = check_hostname)]
    pub hostname: String,

    /// Config file for connection details
    #[arg(short, long, default_value_t = String::from("/etc/olympus/hephaestus/client.conf"))]
    pub config: String,

    /// Show more detail about connection
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Action {
    /// Get a value of a key
    Get {
        /// Specify the name of the key
        #[arg(long)]
        key: String,
    },

    /// Set value to a key
    Set {
        /// Specify the name of the key
        #[arg(long)]
        key: String,

        /// Specify the value for the key
        #[arg(long)]
        value: String,
    },

    /// Remove specified key
    RemKey {
        /// Specify the name of the key
        #[arg(long)]
        key: String,
    },

    /// Remove path
    RemPath {
        /// Specify the name of the key
        #[arg(long)]
        key: String,
    },

    /// List keys
    ListKeys {
        /// Specify the name of the key
        #[arg(long)]
        key: String,
    },

    /// Create new hook
    SetHook {
        /// Key prefix for hook sending
        #[arg(long)]
        prefix: String,

        /// Address where the hook is sent
        #[arg(long)]
        link: String
    },

    /// Check that a hook exists
    GetHook {
        /// Key prefix for hook sending
        #[arg(long)]
        prefix: String,
    },

    /// List hooks
    ListHooks {
        /// Key prefix for hook sending
        #[arg(long)]
        prefix: String,
    },

    /// Remove existing hook
    RemHook {
        #[arg(long)]
        /// Key prefix for hook sending
        prefix: String,

        /// Address where the hook is sent
        #[arg(long)]
        link: String,
    },

    /// Suspend file writing for database log
    SuspendLog,

    /// Resule file writing for database log
    ResumeLog
}

fn check_hostname(s: &str) -> Result<String, String> {
    if !s.starts_with("http://") && !s.starts_with("https://") && !s.starts_with("cfg://") {
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
