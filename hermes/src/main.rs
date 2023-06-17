use std::process::exit;

use arg::Mode;
use clap::Parser;

mod arg;
mod cli;
mod server;
mod shell;

fn main() {
    let args = arg::Parameters::parse();

    // Start runtime
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async move {
        match args.mode {
            Mode::Cli(config) => match cli::main_async(config).await {
                Ok(rc) => exit(rc),
                Err(_) => exit(-999),
            },
            Mode::Server { config } => match server::main_async(config).await {
                Ok(rc) => exit(rc),
                Err(_) => exit(-999),
            },
            Mode::Shell { hostname } => match shell::main_async(hostname).await {
                Ok(rc) => exit(rc),
                Err(_) => exit(-999),
            },
        }
    });
}
