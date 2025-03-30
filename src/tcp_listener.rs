use crate::config::Config;
use crate::{debug, log};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn start_tcp_listener(config: Arc<Mutex<Config>>, config_string: Arc<Mutex<String>>) {
    thread::spawn(move || {
        let config = config.clone();

        let port = {
            let cfg = config.lock().unwrap();
            let self_name = &cfg.config_metadata.name;
            cfg.ddns.iter().find(|d| d.name == *self_name).unwrap().port
        };

        let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)) {
            Ok(listener) => listener,
            Err(error) => {
                panic!("TcpListener can't bind to port {port}, {:?}", error);
            }
        };

        log!("Rocking on port {port}!");

        for stream in listener.incoming() {
            debug!("Connection established");

            if let Ok(stream) = stream {
                let reader = BufReader::new(&stream);
                let mut writer = &stream;

                for line in reader.lines().map_while(Result::ok) {
                    let remote_addr = stream.peer_addr().unwrap().ip().to_string();
                    let line = line.as_str();

                    debug!(
                        "Received line: {:?} (l:{}) from {}",
                        line,
                        line.len(),
                        remote_addr
                    );

                    if line.len() == 4 && &line[0..4] == "PING" {
                        let _ = writer.write_all(b"PONG\n");
                        let _ = writer.flush();
                    } else if line.len() >= 10 && &line[0..10] == "GET CONFIG" {
                        let config_str = config_string.lock().unwrap();
                        let _ = writer
                            .write_all(format!("{}\n", config_str.replace("\n", "\\n")).as_bytes());
                        let _ = writer.flush();
                        debug!("Sent config to {}", remote_addr);
                    }
                }
            }
        }
    });
}
