use crate::{config::Config, log, node_connections::NodeConnections, process::Process};
use futures::future::join_all;
use std::sync::{Arc, Mutex};
use tokio::task;

#[derive(Clone)]
pub struct Node {
    alive: bool,
    pub config: Arc<Mutex<Config>>,
    alives: Vec<bool>,
    process: Option<Arc<Mutex<Process>>>,
    pub node_connections: NodeConnections,
    // tick: u8,
    // tick_dir: u8,
}

impl Node {
    pub fn new(config: Arc<Mutex<Config>>) -> Node {
        let alives = vec![false; config.lock().unwrap().ddns.len()];

        Node {
            alive: false,
            config,
            alives,
            process: None,
            node_connections: NodeConnections::new(),
            // tick: 0,
            // tick_dir: 1,
        }
    }

    /// Returns the amount of alive hosts
    pub async fn check_hosts(&mut self) -> u8 {
        let config_metadata_name = self.config.lock().unwrap().config_metadata.name.clone();
        let alives = Arc::new(Mutex::new(0u8));
        let mut handles = Vec::new();

        for (index, host) in self.config.lock().unwrap().ddns.iter().enumerate() {
            if host.name == config_metadata_name {
                continue;
            }

            self.alives[index] = false;
            let host_clone = host.clone();
            let alives_clone = Arc::clone(&alives);
            let mut node_connections = self.node_connections.clone();

            let handle = task::spawn(async move {
                let host_name = host_clone.name.clone();
                let host_priority = host_clone.priority;

                log!(
                    "Checking: {}:{}",
                    if host_clone.preference == 0 {
                        &host_clone.ddns
                    } else {
                        &host_clone.ip
                    },
                    &host_clone.port
                );

                let alive = task::spawn_blocking(move || node_connections.ping(&host_clone))
                    .await
                    .unwrap();

                if alive {
                    log!(
                        "-> Alive: host \"{}\" with priority {}",
                        host_name,
                        host_priority,
                    );
                    let mut count = alives_clone.lock().unwrap();
                    *count += 1;
                } else {
                    log!(
                        "-> Host \"{}\" with priority {} is dead",
                        host_name,
                        host_priority,
                    );
                }
            });

            handles.push(handle);
        }

        join_all(handles).await;

        let alives = *alives.lock().unwrap();
        alives
    }

    fn spawn(&mut self) {
        let process = Process::new(&self.config.lock().unwrap());
        self.process = Some(Arc::new(Mutex::new(process)));
    }

    /// Check for config updates and update
    #[allow(dead_code)]
    async fn check_config_diffs(&mut self) -> bool {
        let c = self.config.lock().unwrap().clone();
        'outer: for host in c.ddns.iter().enumerate() {
            if host.1.name == c.config_metadata.name {
                continue;
            }

            if !self.alives[host.0] {
                continue;
            }

            // Connection
            let connection_mutex = if let Some(conn) = self
                .node_connections
                .get_node_connection(host.1.name.clone())
            {
                conn
            } else if let Some(conn) = self.node_connections.create_node_connection(host.1) {
                conn
            } else {
                // If no connection can be established, continue the outer loop
                continue 'outer;
            };

            log!("Checking for config updates");
            let mut connection = connection_mutex.lock().unwrap();
            let _ = connection.update_config(self.config.clone());
        }
        false
    }

    pub async fn heartbeat(&mut self) {
        log!("\n====> Heartbeat");

        let alives = self.check_hosts().await;
        log!("\nAll hosts checked!");

        log!("-> Alives: {}", alives);
        if !self.alive
            && (alives == 0 || {
                // There are nodes alive with less priority
                let config_guard = self.config.lock().unwrap();
                assert!(config_guard.ddns.len() == self.alives.len());
                let local_priority = config_guard
                    .ddns
                    .iter()
                    .find(|d| d.name == config_guard.config_metadata.name)
                    .map(|d| d.priority)
                    .unwrap_or(0);

                !config_guard
                    .ddns
                    .iter()
                    .zip(self.alives.iter())
                    .any(|(host, &alive)| alive && host.priority > local_priority)
            })
        {
            log!("\n-> Node switching to alive");
            self.alive = true;
            self.spawn();
        } else {
            // Hosts alive
            // First check configs and then kill or otherwise?
            let config_guard = self.config.lock().unwrap();
            let local_priority = config_guard
                .ddns
                .iter()
                .find(|d| d.name == config_guard.config_metadata.name)
                .map(|d| d.priority)
                .unwrap_or(0);

            if self.process.is_some()
                && config_guard
                    .ddns
                    .iter()
                    .any(|d| d.priority > local_priority)
            {
                // Clean up
                self.alive = false;
                if let Some(p) = &self.process {
                    {
                        let mut process = p.lock().unwrap();
                        process.kill();
                    }
                    self.process = None;
                }
            }
        }

        // if alives != 0 && self.tick % 5 == 0 {
        // self.check_config_diffs().await;
        // }

        // if self.tick == 0 {
        //     self.tick_dir = 1;
        // } else if self.tick == 5 {
        //     self.tick_dir = 0;
        // }
        // if self.tick_dir == 1 {
        //     self.tick += 1
        // } else {
        //     self.tick -= 1
        // };
        log!("====> Hearbeat end");
    }
}
