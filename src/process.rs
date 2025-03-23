use crate::{config::Config, log};

pub struct Process {
    pub child: std::process::Child,
}

impl Process {
    pub fn new(cfg: &Config) -> Process {
        let args: Vec<&str> = cfg.execution.instructions.split(" ").collect();
        let child = std::process::Command::new(args[0])
            .args(&args[1..])
            .spawn()
            .expect("Couldn't spawn the process.");

        Process { child }
    }

    pub fn kill(&mut self) {
        log!("Killing process {}", self.child.id());
        self.child.kill().expect("!kill");
    }
}
