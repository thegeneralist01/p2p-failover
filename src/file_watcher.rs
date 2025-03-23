use crate::config::Config;
use crate::parser::Parser;
use crate::{debug, log};
use notify::{RecommendedWatcher, Watcher};
use std::sync::{Arc, Mutex};
use std::thread;
use std::{fs::File, path::Path};

pub fn start_file_watcher(config: Arc<Mutex<Config>>, config_string: Arc<Mutex<String>>) {
    thread::spawn(move || {
        let config_path = std::env::var("P2P_CONFIG_PATH")
            .unwrap_or_else(|_| "p2p-failover.config.yaml".to_string());

        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher: RecommendedWatcher =
            match notify::Watcher::new(tx, notify::Config::default()) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("Failed to create watcher: {:?}", e);
                    return;
                }
            };

        if let Err(e) = watcher.watch(
            Path::new(&config_path.clone()),
            notify::RecursiveMode::NonRecursive,
        ) {
            eprintln!("Failed to watch config file: {:?}", e);
            return;
        }

        // Block forever, printing out events as they come in
        for res in rx {
            if let Err(e) = res {
                log!("watch error: {:?}", e);
                continue;
            }

            let event = res.unwrap();
            debug!("event: {:?}", event);

            if let notify::EventKind::Modify(_) = event.kind {
                // Refresh config
                let config_file = get_file(&config_path);
                let mut p = Parser::new(config_file);
                let cfg = p.parse(Some(config_string.clone()));
                if let Ok(cfg) = cfg {
                    let mut config_guard = config.lock().unwrap();
                    *config_guard = cfg;
                    log!("Config updated: {:#?}", config_guard);
                }
            }
        }
    });
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
