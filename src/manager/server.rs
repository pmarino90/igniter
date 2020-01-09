use std::collections::hash_map::{Entry, HashMap};
use std::fs;
use std::fs::File;
use std::path::Path;
use std::thread;
use std::time::Duration;

use daemonize::Daemonize;

use crate::config;
use crate::manager;
use crate::rpc;

struct State {
    processes: HashMap<String, manager::Process>,
}

impl State {
    pub fn new() -> State {
        State {
            processes: HashMap::new(),
        }
    }
}

struct Monitor {
    server: rpc::Server,
    state: State,
}

impl Monitor {
    fn run(&mut self) {
        loop {
            for (_name, process) in self.state.processes.iter_mut() {
                let status = process.status();
                println!(
                    "[{}] - status {:?},  desired: {:?}",
                    process.name, status, process.desired_status
                );
                let _ = process.try_reconciliate();
            }

            if let Ok(Some(undecoded_msg)) = self.server.try_receive() {
                if let Ok(msg) = undecoded_msg.decode() {
                    match msg {
                        rpc::Message::Start(name, config) => {
                            let entry = self.state.processes.entry(name.clone());

                            if let Entry::Vacant(_) = entry {
                                let config::Process { program, args } = config.into_owned();
                                let mut process = manager::Process::new(name, program, args);
                                if process.start().is_ok() {
                                    entry.or_insert(process);
                                }
                            }
                        }

                        rpc::Message::Stop(name) => {
                            let entry = self.state.processes.entry(name);
                            if let Entry::Occupied(_) = entry {
                                entry.and_modify(|v| {
                                    let _ = v.stop();
                                });
                            }
                        }

                        rpc::Message::Quit => break,
                        rpc::Message::Status => {}
                    }
                }
            }

            thread::sleep(Duration::from_millis(100));
        }
    }
}

pub fn start(config_dir: &Path, daemonize: bool) {
    let pid_file = if daemonize {
        let pid_file = config_dir.join("daemon.pid");

        let stdout = File::create(config_dir.join("stdout.log")).unwrap();
        let stderr = File::create(config_dir.join("stderr.log")).unwrap();

        let daemonize = Daemonize::new()
            .pid_file(&pid_file)
            .stdout(stdout)
            .stderr(stderr);

        daemonize.start().expect("couldn't create daemon.");

        Some(pid_file)
    } else {
        None
    };

    let server = rpc::Server::new(config_dir).unwrap();

    let mut monitor = Monitor {
        server,
        state: State::new(),
    };
    monitor.run();

    if let Some(file) = pid_file {
        fs::remove_file(file).expect("can't remove pid file");
    }

    println!("finished.")
}
