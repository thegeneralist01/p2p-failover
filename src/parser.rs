use std::{
    io::Read,
    sync::{Arc, Mutex},
};

use crate::config::Config;

pub struct Parser<R: Read> {
    src: R,
}

impl<R: Read> Parser<R> {
    pub fn new(src: R) -> Parser<R> {
        Parser { src }
    }

    pub fn parse(
        &mut self,
        config_str: Option<Arc<Mutex<String>>>,
    ) -> Result<Config, std::io::Error> {
        let mut contents = String::new();
        self.src.read_to_string(&mut contents)?;

        // parse
        let cfg: Config = serde_yaml::from_str(&contents).unwrap();
        if let Some(config_str) = config_str {
            *config_str.lock().unwrap() = contents;
        }

        Ok(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parser() {
        let yaml = r#"
nodes:
- name: test
  ip: 127.0.0.1
  port: 8080
  priority: 100
  last_updated: 2024-03-20 00:00:00 UTC
config_metadata:
  name: test
  last_updated: 2024-03-20 00:00:00 UTC
execution:
  instructions: ./test.sh
  last_updated: 2024-03-20 00:00:00 UTC
"#;
        let mut parser = Parser::new(Cursor::new(yaml));
        let result = parser.parse(None);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.nodes.len(), 1);
        assert_eq!(config.nodes[0].ip, "127.0.0.1");
        assert_eq!(config.nodes[0].priority, 100);
        assert_eq!(config.nodes[0].name, "test");
        assert_eq!(config.nodes[0].name, config.config_metadata.name);
    }
}
