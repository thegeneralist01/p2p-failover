use crate::timestamp::Timestamp;
use serde::{Deserialize, Serialize};

// Config
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderNode {
    pub name: String,
    pub ddns: String,
    pub ip: String,
    pub port: u32,
    pub preference: u8,
    pub priority: u32,
    pub last_updated: Timestamp,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ConfigMetadata {
    pub name: String,
    pub last_updated: Timestamp,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ExecutionInstructions {
    pub instructions: String,
    pub last_updated: Timestamp,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub ddns: Vec<ProviderNode>,
    pub config_metadata: ConfigMetadata,
    pub execution: ExecutionInstructions,
}

impl Config {
    pub fn write(&self) {
        let config_path = std::env::var("P2P_CONFIG_PATH")
            .unwrap_or_else(|_| "p2p-failover.config.yaml".to_string());

        let s = serde_yaml::to_string(&self).unwrap();
        match std::fs::write(config_path, s) {
            Ok(_) => (),
            Err(e) => eprintln!("Failed to write config file: {:?}", e),
        }
    }
}
