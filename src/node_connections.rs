use anyhow::{bail, Result};
use std::{
    io::{BufRead, BufReader},
    sync::mpsc,
    thread,
    time::Duration,
};
#[allow(unused_imports)]
use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
};

use crate::{
    config::{Config, ProviderNode},
    debug, log,
    parser::Parser,
    timestamp::Timestamp,
};

#[derive(Debug)]
pub struct NodeInfo {
    pub target_name: String,
    pub target: String,
    pub port: u32,
    stream: Option<TcpStream>,
}

impl NodeInfo {
    pub fn new(
        target_name: String,
        target: String,
        port: u32,
        stream: Option<TcpStream>,
    ) -> NodeInfo {
        NodeInfo {
            target_name,
            target,
            port,
            stream,
        }
    }

    pub fn update_config(&mut self, config_self_mutex: Arc<Mutex<Config>>) -> Result<()> {
        if let Some(ref mut stream) = self.stream {
            let (tx, rx) = mpsc::channel();
            let read_stream = stream.try_clone().unwrap();

            thread::spawn(move || {
                let mut reader = BufReader::new(read_stream);
                let mut response = String::new();
                match reader.read_line(&mut response) {
                    Ok(_) => {
                        debug!("Inside: Received response: {:?}", response);
                        tx.send(response.trim().to_string()).unwrap_or_default();
                    }
                    Err(e) => {
                        debug!("Error reading response: {:?}", e);
                        tx.send(String::new()).unwrap_or_default();
                    }
                }
            });

            stream.write_all(b"GET CONFIG\n")?;

            let s = match rx.recv_timeout(Duration::from_secs(2)) {
                Ok(response) => {
                    debug!("Received response: {:?}", response);
                    response.replace("\\n", "\n")
                }
                Err(e) => {
                    debug!("Timeout waiting for response: {:?}", e);
                    String::new()
                }
            };
            if s.is_empty() {
                debug!("Empty response: {:?}", s);
                bail!("No response");
            }

            let cfg: Config = match serde_yaml::from_str(&s) {
                Ok(cfg) => cfg,
                Err(e) => {
                    debug!("Error parsing config: {:?}", e);
                    bail!(e);
                }
            };

            let mut config_self = config_self_mutex.lock().unwrap();
            if config_self.config_metadata.last_updated > cfg.config_metadata.last_updated {
                debug!("Local config is newer, aborting");
                return Ok(());
            }

            // Update the config
            // Execution instructions
            config_self.execution.instructions = cfg.execution.instructions;

            let node_self_name = config_self.config_metadata.name.clone();

            // Add new Nodes (that do not exist in our config, but exist in the other config)
            for node in &cfg.nodes {
                if node.name == node_self_name {
                    continue;
                }
                if !config_self.nodes.iter().any(|d| d.name == node.name) {
                    config_self.nodes.push(node.clone());
                }
            }

            config_self.config_metadata.last_updated = cfg.config_metadata.last_updated.clone();
            // Wondering if we should update the last updated
            config_self
                .nodes
                .iter_mut()
                .find(|d| d.name == node_self_name)
                .unwrap()
                .last_updated = Timestamp::now();

            config_self.write();
            log!("Updated config successfully");

            return Ok(());
        }

        debug!("No stream for {}", self.target_name);
        bail!("No stream");
    }
}

#[derive(Clone)]
pub struct NodeConnections {
    connections: Vec<Arc<Mutex<NodeInfo>>>,
}

impl Default for NodeConnections {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeConnections {
    pub fn new() -> NodeConnections {
        NodeConnections {
            connections: vec![],
        }
    }

    pub fn get_node_connection(&self, node_name: String) -> Option<Arc<Mutex<NodeInfo>>> {
        for connection in &self.connections {
            let conn = connection.lock().unwrap();
            if conn.target_name == node_name && conn.stream.is_some() {
                return Some(connection.clone());
            }
        }
        None
    }

    pub fn get_alive_connections(&self) -> &Vec<Arc<Mutex<NodeInfo>>> {
        &self.connections
    }

    pub fn ping(&mut self, node: &ProviderNode) -> bool {
        let target = node.ip.clone();
        let mut connection: Option<Arc<Mutex<NodeInfo>>> =
            self.get_node_connection(node.name.clone());

        if connection.is_none()
            || (connection.is_some() && !is_connection_alive(connection.clone().unwrap()))
        {
            if connection.is_some() {
                self.remove_node_connection(node.name.clone());
            }
            connection = self.create_node_connection(node);
            if connection.is_none() {
                return false;
            }
        }

        let connection = connection.unwrap();
        let connection_guard = connection.lock().unwrap();

        if connection_guard.stream.is_none() {
            return false;
        }

        let mut stream = connection_guard
            .stream
            .as_ref()
            .unwrap()
            .try_clone()
            .unwrap();

        let (tx, rx) = mpsc::channel();
        let read_stream = stream.try_clone().unwrap();

        thread::spawn(move || {
            let mut reader = BufReader::new(read_stream);
            let mut response = String::new();
            match reader.read_line(&mut response) {
                Ok(_) => {
                    let is_pong = response.trim() == "PONG";
                    tx.send(is_pong as i8).unwrap_or_default();
                }
                Err(e) => {
                    debug!("Error reading response (in fn `ping`): {:?}", e);
                    tx.send(-1).unwrap_or_default();
                }
            }
        });

        // Write PING
        let _ = stream.write_all(b"PING\n");

        let _ = stream.flush();

        let reply = rx.recv_timeout(Duration::from_secs(2)).unwrap_or_default();
        if reply == -1 {
            self.remove_node_connection(target.clone());
        }
        reply == 1
    }

    pub fn create_node_connection(&mut self, node: &ProviderNode) -> Option<Arc<Mutex<NodeInfo>>> {
        let stream = TcpStream::connect_timeout(
            &std::net::SocketAddr::new(node.ip.clone().parse().unwrap(), node.port as u16),
            Duration::from_millis(500),
        );

        match stream {
            Ok(stream) => {
                let connection = Arc::new(Mutex::new(NodeInfo::new(
                    node.name.clone(),
                    node.ip.clone(),
                    node.port,
                    Some(stream),
                )));

                self.connections.push(connection.clone());
                Some(connection.clone())
            }

            Err(error) => {
                if error.kind() != std::io::ErrorKind::ConnectionRefused {
                    log!("-> Problem creating the stream: {:?}", error);
                }
                None
            }
        }
    }

    pub fn remove_node_connection(&mut self, target_name: String) {
        if let Some(pos) = self
            .connections
            .iter()
            .position(|conn| conn.lock().unwrap().target_name == target_name)
        {
            self.connections.remove(pos);
        }
    }

    pub fn confirm(&mut self, source: &str, is_ip: bool) -> Option<String> {
        for connection in &self.connections {
            let conn = connection.lock().unwrap();
            if conn.stream.is_none() {
                continue;
            }

            let mut stream = conn.stream.as_ref().unwrap();
            stream
                .write_all(format!("CONFIRM:{}:{}\n", is_ip as u8, source).as_bytes())
                .unwrap();

            let reader = BufReader::new(stream);

            let sis_ip = is_ip.to_string();

            for line in reader.lines() {
                if line.is_err() {
                    log!("Error reading line: {:?}", line.err());
                    return None;
                };
                // Template: CONFIRM:_:_:bool
                // bool is 0/1
                let line = line.unwrap();
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() != 4 {
                    log!("Invalid response: {}", line);
                    return None;
                }

                if parts[0] != "CONFIRM" {
                    log!("Invalid response: {}", line);
                    return None;
                }

                if parts[1] != sis_ip {
                    log!("Invalid response: {}", line);
                    return None;
                }

                if parts[2] != source {
                    log!("Invalid response: {}", line);
                    return None;
                }

                if parts[3] == "1" {
                    return Some(conn.target_name.clone());
                }
            }
        }

        None
    }

    pub fn get_config_for(
        &mut self,
        source: &str,
        target_name: String,
    ) -> Option<ProviderNode> {
        for connection in &self.connections {
            let conn = connection.lock().unwrap();
            if conn.stream.is_none() || conn.target_name != target_name {
                continue;
            }

            let mut stream = conn.stream.as_ref().unwrap();
            stream.write_all(b"GET CONFIG\n").unwrap();

            let reader = BufReader::new(stream);

            for line in reader.lines() {
                if line.is_err() {
                    log!("Error reading line: {:?}", line.err());
                    continue;
                };
                let line = line.unwrap();
                let mut parser = Parser::new(line.as_bytes());

                let cfg = match parser.parse(None) {
                    Ok(cfg) => cfg,
                    Err(_) => {
                        stream.write_all(b"AUTH FAIL: BAD CONFIG\n").unwrap();
                        continue;
                    }
                };

                if let Some(provider) =  cfg.nodes.iter().find(|d| d.ip.clone() == source) {
                    return Some(provider.clone());
                } else {
                    return None;
                }
            }
        }
        None
    }
}

fn is_connection_alive(connection: Arc<Mutex<NodeInfo>>) -> bool {
    let connection_guard = connection.lock().unwrap();
    if connection_guard.stream.is_none() {
        return false;
    }

    let mut stream = connection_guard.stream.as_ref().unwrap();
    match stream.write(&[]) {
        Ok(_) => true,
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => true,
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => false,
        Err(_) => false,
    }
}
