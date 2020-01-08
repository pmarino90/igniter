use std::fs;
use std::fs::File;
use std::path::Path;
use std::thread;
use std::time::Duration;

use daemonize::Daemonize;

use crate::manager;
use crate::rpc;

struct Monitor {
    server: rpc::Server,
    processs: Vec<manager::Process>,
}

impl Monitor {
    fn run(&mut self) {
        loop {
            for process in self.processs.iter_mut() {
                let desired_status = manager::Status::Running;
                let _ = process.try_reconciliate(&desired_status);
            }

            if let Ok(Some(msg)) = self.server.try_receive() {
                match msg {
                    rpc::Message::Start(_, _) => {}
                    rpc::Message::Stop(_process_name) => {}
                    rpc::Message::Quit => break,
                    rpc::Message::Status => {}
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
        processs: vec![],
    };
    monitor.run();

    if let Some(file) = pid_file {
        fs::remove_file(file).expect("can't remove pid file");
    }

    println!("finished.")
}
