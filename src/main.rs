use anyhow::Result;
use p2p_failover::{file_watcher, node::Node, parser::Parser, tcp_listener};
use std::{
    fs::File,
    sync::{Arc, Mutex},
    thread, time,
};

#[tokio::main]
async fn main() -> Result<()> {
    let config_path =
        std::env::var("P2P_CONFIG_PATH").unwrap_or_else(|_| "p2p-failover.config.yaml".to_string());

    let config_file = get_file(&config_path);
    let mut p = Parser::new(config_file);

    let config_string = Arc::new(Mutex::new(String::new()));
    let config = {
        let cfg = p.parse(Some(config_string.clone()))?;
        Arc::new(Mutex::new(cfg))
    };

    let mut node = Node::new(config.clone());

    file_watcher::start_file_watcher(config.clone(), config_string.clone());
    tcp_listener::start_tcp_listener(config.clone(), config_string.clone());

    loop {
        node.heartbeat().await;
        thread::sleep(time::Duration::from_secs(1))
    }
}

fn get_file(filename: &String) -> File {
    match File::open(filename) {
        Ok(file) => file,
        Err(error) => {
            // if file not created
            if error.kind() == std::io::ErrorKind::NotFound {
                match File::create(filename) {
                    Ok(file) => file,
                    Err(error) => panic!("Problem creating the file: {:?}", error),
                }
            } else {
                panic!("Problem opening the file: {:?}", error);
            }
        }
    }
}
