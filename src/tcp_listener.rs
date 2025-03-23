use crate::config::Config;
use crate::{debug, log};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn start_tcp_listener(config: Arc<Mutex<Config>>, config_string: Arc<Mutex<String>>) {
    thread::spawn(move || {
        //let trustkey_path =
        //std::env::var("P2P_TRUSTKEY_PATH").unwrap_or_else(|_| ".p2p-trustkey".to_string());
        //let mut trustkey = String::new();
        //let _ = get_file(&trustkey_path).read_to_string(&mut trustkey);

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
            debug!("CONNECTION established");

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
                    // else if line.len() > 8 && &line[0..8] == "CONFIRM:" {
                    //     // Template: CONFIRM:is_ip:source
                    //     // is_ip is either 0 or 1
                    //     let parts = line.split(":").collect::<Vec<&str>>();
                    //     if parts.len() != 3 || !(parts[1] == "0" || parts[1] == "1") {
                    //         debug!("CONFIRM FAIL: BAD REQUEST");
                    //         let _ = writer.write_all(b"AUTH FAIL: BAD REQUEST\n");
                    //         continue;
                    //     }
                    //
                    //     let is_ip = parts[1] == "1";
                    //     let source = parts[2];
                    //
                    //     let config_guard = config.lock().unwrap();
                    //     let found = config_guard.ddns.iter().any(|d| {if is_ip {&d.ip} else {&d.ddns}} == source);
                    //     let _ = writer
                    //         .write_all({
                    //             if found {
                    //                 b"1\n"
                    //             } else {
                    //                 b"0\n"
                    //             }
                    //         });
                    // }
                    // else if line.len() >= 8 && &line[0..8] == "AUTH REQ" {
                    //     // Template: AUTH:source:port:trustkey:redirect_node
                    //     let parts = line.split(":").collect::<Vec<&str>>();
                    //     if parts.len() < 4 || parts.len() > 5 {
                    //         writer.write_all(b"AUTH FAIL: BAD REQUEST\n").unwrap();
                    //         continue;
                    //     }
                    //
                    //     let source = parts[2];
                    //     let _source_port = match parts[3].parse::<u32>() {
                    //         Ok(p) => p,
                    //         Err(_) => continue,
                    //     };
                    //     let is_ip = source.chars().all(|c: char| c == '.' || c.is_ascii_digit());
                    //
                    //     // Check if other Nodes have it
                    //     {
                    //         let mut node_guard = node.lock().unwrap();
                    //         let node_confirmed = node_guard.node_connections.confirm(source, is_ip);
                    //         if let Some(node_confirmed) = node_confirmed {
                    //             let provider = node_guard.node_connections.get_config_for(
                    //                 source,
                    //                 is_ip,
                    //                 node_confirmed,
                    //             );
                    //             if let Some(provider) = provider {
                    //                 let mut config_guard = config.lock().unwrap();
                    //                 config_guard.ddns.push(provider);
                    //             }
                    //         }
                    //     };
                    //
                    //     // DDNS; Verification
                    //     if is_ip && remote_addr != source {
                    //         writer.write_all(b"AUTH FAIL: SOURCE MISMATCH\\nn").unwrap();
                    //         continue;
                    //     }
                    //
                    //     let request_trustkey = parts[4];
                    //     if request_trustkey != trustkey {
                    //         writer.write_all(b"AUTH FAIL: TRUSTKEY MISMATCH\n").unwrap();
                    //         continue;
                    //     }
                    //
                    //     let config_guard = config.lock().unwrap();
                    //     let ddns = config_guard.ddns.iter().find(|d| d.name == source);
                    //     if ddns.is_some() {
                    //         writer.write_all(b"AUTH SUCCESS: ALREADY EXISTS\n").unwrap();
                    //         continue;
                    //     }
                    //
                    //     // TODO: A) This. Search for `TODO: A)`
                    //     verifications.push(PendingVerification {
                    //         source: source.to_string(),
                    //         remote_addr,
                    //         redirect_node: {
                    //             if parts.len() >= 5 {
                    //                 parts[5]
                    //             } else {
                    //                 ""
                    //             }
                    //         }
                    //         .to_string(),
                    //         is_ip,
                    //     });
                    //
                    //     writer.write_all(b"GET CONFIG\n").unwrap();
                    // }
                    // else if line.len() > 12 && &line[0..12] == "AUTH PENDING" {
                    //     // Template: AUTH PENDING:config
                    //     if !verifications.iter().any(|v| remote_addr == v.remote_addr) {
                    //         writer.write_all(b"AUTH FAIL: NOT PENDING\n").unwrap();
                    //         continue;
                    //     };
                    //
                    //     let config_incoming = &line[13..line.len()];
                    //     let mut parser_incoming = Parser::new(config_incoming.as_bytes());
                    //
                    //     let config_incoming = match parser_incoming.parse(None) {
                    //         Ok(cfg) => cfg,
                    //         Err(_) => {
                    //             writer.write_all(b"AUTH FAIL: BAD CONFIG\n").unwrap();
                    //             continue;
                    //         }
                    //     };
                    //
                    //     let ddns_incoming = match config_incoming
                    //         .ddns
                    //         .iter()
                    //         .find(|d| d.name == config_incoming.config_metadata.name)
                    //     {
                    //         Some(d) => d,
                    //         None => {
                    //             writer
                    //                 .write_all(b"AUTH FAIL: SELF ABSENT IN DDNS\n")
                    //                 .unwrap();
                    //             continue;
                    //         }
                    //     };
                    //
                    //     let mut config_guard = config.lock().unwrap();
                    //     config_guard.ddns.push(ddns_incoming.clone());
                    // }
                }
            }
        }
    });
}
