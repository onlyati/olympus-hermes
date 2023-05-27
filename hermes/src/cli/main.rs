// External dependencies
use clap::Parser;
use hermes::hermes_client::HermesClient;
use hermes::{Empty, Hook, HookCollection, Key, KeyList, Pair};
use std::process::exit;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use tonic::{Request, Response, Status};

// Internal dependencies
mod arg;
use arg::{Action, Args};

// Generate structs for gRPC
mod hermes {
    tonic::include_proto!("hermes");
}

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async move {
        match main_async().await {
            Ok(rc) => exit(rc),
            Err(_) => exit(-999),
        }
    });
}

async fn main_async() -> Result<i32, Box<dyn std::error::Error>> {
    let args = Args::parse();

    let start = std::time::Instant::now();

    // Try to connect to gRPC server
    let grpc_channel = create_grpc_channel(args.clone()).await;

    let mut grpc_client = HermesClient::new(grpc_channel);

    let mut final_rc = 0;

    match &args.action {
        // GET action
        Action::Get { key } => {
            let response: Result<Response<Pair>, Status> = grpc_client
                .get(Request::new(Key { key: key.clone() }))
                .await;

            match response {
                Ok(resp) => {
                    let resp = resp.into_inner();
                    println!("{}", resp.value);
                }
                Err(e) => {
                    eprintln!("Failed request: {}", e.message());
                    final_rc = 4;
                }
            }
        }
        // SET action
        Action::Set { key, value } => {
            let response: Result<Response<Empty>, Status> = grpc_client
                .set(Request::new(Pair {
                    key: key.clone(),
                    value: value.clone(),
                }))
                .await;

            if let Err(e) = response {
                eprintln!("Failed request: {}", e.message());
                final_rc = 4;
            }
        }
        // REMKEY action
        Action::RemKey { key } => {
            let response: Result<Response<Empty>, Status> = grpc_client
                .delete_key(Request::new(Key { key: key.clone() }))
                .await;

            if let Err(e) = response {
                eprintln!("Failed request: {}", e.message());
                final_rc = 4;
            }
        }
        // REMPATH action
        Action::RemPath { key } => {
            let response: Result<Response<Empty>, Status> = grpc_client
                .delete_path(Request::new(Key { key: key.clone() }))
                .await;

            if let Err(e) = response {
                eprintln!("Failed request: {}", e.message());
                final_rc = 4;
            }
        }
        // LIST action
        Action::ListKeys { key } => {
            let response: Result<Response<KeyList>, Status> = grpc_client
                .list_keys(Request::new(Key { key: key.clone() }))
                .await;

            match response {
                Ok(resp) => {
                    let key_list = resp.into_inner();

                    for key in key_list.keys {
                        println!("{}", key);
                    }
                }
                Err(e) => {
                    eprintln!("Failed request: {}", e.message());
                    final_rc = 4;
                }
            }
        }
        // TRIGGER action
        Action::Trigger { key, value } => {
            let response: Result<Response<Empty>, Status> = grpc_client
                .trigger(Request::new(Pair {
                    key: key.clone(),
                    value: value.clone(),
                }))
                .await;

            if let Err(e) = response {
                eprintln!("Failed request: {}", e.message());
                final_rc = 4;
            }
        }
        // GETHOOK action
        Action::GetHook { prefix } => {
            let response: Result<Response<Hook>, Status> = grpc_client
                .get_hook(Request::new(Key {
                    key: prefix.clone(),
                }))
                .await;

            match response {
                Ok(resp) => {
                    let hook = resp.into_inner();
                    let links = hook.links;

                    match links {
                        Some(links) => {
                            for link in links.links {
                                println!("{}", link);
                            }
                        }
                        None => {
                            eprintln!("Failed request: No hook found");
                            final_rc = 4;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed request: {}", e.message());
                    final_rc = 4;
                }
            }
        }
        // SETHOOK action
        Action::SetHook { prefix, link } => {
            let response: Result<Response<Empty>, Status> = grpc_client
                .set_hook(Request::new(Pair {
                    key: prefix.clone(),
                    value: link.clone(),
                }))
                .await;

            if let Err(e) = response {
                eprintln!("Failed request: {}", e.message());
                final_rc = 4;
            }
        }
        Action::RemHook { prefix, link } => {
            let response: Result<Response<Empty>, Status> = grpc_client
                .delete_hook(Request::new(Pair {
                    key: prefix.clone(),
                    value: link.clone(),
                }))
                .await;

            if let Err(e) = response {
                eprintln!("Failed request: {}", e.message());
                final_rc = 4;
            }
        }
        Action::ListHooks { prefix } => {
            let response: Result<Response<HookCollection>, Status> = grpc_client
                .list_hooks(Request::new(Key {
                    key: prefix.clone(),
                }))
                .await;

            match response {
                Ok(resp) => {
                    let collection = resp.into_inner();

                    for hook in collection.hooks {
                        let mut line = String::from(hook.prefix);
                        match hook.links {
                            Some(link_collection) => {
                                for link in link_collection.links {
                                    line += " ";
                                    line += &link[..];
                                }
                            },
                            None => {
                                eprintln!("Failed request: No hook found");
                                final_rc = 4;
                            }
                        }
                        println!("{}", line);
                    }
                }
                Err(e) => {
                    eprintln!("Failed request: {}", e.message());
                    final_rc = 4;
                }
            }
        }
        Action::SuspendLog => {
            let response: Result<Response<Empty>, Status> =
                grpc_client.suspend_log(Request::new(Empty {})).await;

            if let Err(e) = response {
                eprintln!("Failed request: {}", e.message());
                final_rc = 4;
            }
        }
        Action::ResumeLog => {
            let response: Result<Response<Empty>, Status> =
                grpc_client.resume_log(Request::new(Empty {})).await;

            if let Err(e) = response {
                eprintln!("Failed request: {}", e.message());
                final_rc = 4;
            }
        }
    }

    let elapsed = start.elapsed();
    print_verbose(&args, format!("Measured runtime: {:?}", elapsed));

    return Ok(final_rc);
}

/// Print text only, when verbose flag is set
fn print_verbose<T: std::fmt::Display>(args: &Args, text: T) {
    if args.verbose {
        println!("> {}", text);
    }
}

/// Create a new gRPC channel which connection to Hephaestus
async fn create_grpc_channel(args: Args) -> Channel {
    if !args.hostname.starts_with("cfg://") {
        print_verbose(&args, "Not cfg:// procotll is given");
        return Channel::from_shared(args.hostname.clone())
            .unwrap()
            .connect()
            .await
            .unwrap();
    }

    let host = args.hostname[6..].to_string();

    print_verbose(
        &args,
        format!(
            "cfg:// is specified, will be looking for in {} for {} settings",
            host, args.config
        ),
    );

    let config = match onlyati_config::read_config(&args.config[..]) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read config: {}", e);
            std::process::exit(2);
        }
    };

    let addr = match config.get(&format!("node.{}.address", host)) {
        Some(a) => a.clone(),
        None => {
            eprintln!("No address is found for '{}' in config", host);
            std::process::exit(2);
        }
    };

    let ca = config.get(&format!("node.{}.ca_cert", host));
    let domain = config.get(&format!("node.{}.domain", host));

    print_verbose(&args, format!("{:?}, {:?}", ca, domain));

    if ca.is_some() && domain.is_some() {
        let pem = match tokio::fs::read(ca.unwrap()).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to read {}: {}", ca.unwrap(), e);
                std::process::exit(2);
            }
        };
        let ca = Certificate::from_pem(pem);

        let tls = ClientTlsConfig::new()
            .ca_certificate(ca)
            .domain_name(domain.unwrap());

        return Channel::from_shared(addr)
            .unwrap()
            .tls_config(tls)
            .unwrap()
            .connect()
            .await
            .unwrap();
    } else {
        return Channel::from_shared(addr).unwrap().connect().await.unwrap();
    }
}
