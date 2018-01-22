extern crate config;
extern crate serde;
extern crate serde_json;
extern crate ctrlc;
extern crate nix;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
#[macro_use] 
extern crate prettytable;

use clap::{Arg, App, SubCommand};
use std::process::{Command, Child, exit};
use std::fs::File;
use std::io::prelude::*;
use std::env::home_dir;
use config::{FileFormat};
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Process {
    name: String,
    cmd: String,
    #[serde(default)]
    pid: i32,
    #[serde(default)]
    child_pid: i32,
    #[serde(default)]
    args: Vec<Vec<String>>,
    #[serde(default)]
    retries: i32,
    #[serde(default)]
    max_retries: i32,
}

#[derive(Debug, Deserialize)]
struct Settings {
    process: Vec<Process>,
}

impl Process {
    fn set_pid(&mut self, pid: i32) {
        self.pid = pid;
    }

    fn set_child_pid(&mut self, pid: i32) {
        self.child_pid = pid;
    }

    fn increment_retries(&mut self) {
        self.retries = self.retries + 1;
    }
}

fn start_processes() {
    println!("Starting processes found in .igniterc");
    let settings = read_settings();
    let processes = settings.process;

    for p in processes {
        let data = serde_json::to_string(&p).unwrap();

        println!("Name: {}", p.name);
        println!("Command: {}", p.cmd);

        let child = Command::new(std::env::current_exe().unwrap())
        .args(&[String::from("monitor"), data.clone()])
        .spawn()
        .expect("Error while starting command");

        println!("Manager process started with PID: {}", child.id());
    }
}

fn save_process_file(name: String, data: String) {
    let mut file = File::create(build_process_filename(name)).unwrap();
    file.write_all(data.as_bytes()).unwrap();
}

fn delete_process_file(pid: i32) {
    std::fs::remove_file(build_process_filename(format!("{}", pid))).unwrap();
}

fn build_process_filename(name: String) -> String {
    let base_path = format!("{}/.igniter/procs", home_dir().unwrap().display());

    std::fs::create_dir_all(base_path.clone()).unwrap();
    format!("{}/{}.json",base_path, name)
}

fn read_process_file(path: String) -> Process {
    println!("Reading process file: {}", path);

    if let Ok(mut file) = File::open(path) {
        let mut content = String::new();
        
        file.read_to_string(&mut content).unwrap();
        return serde_json::from_str(content.as_str()).unwrap();
    }
    panic!("No process file found");
}

fn get_active_processes() -> Vec<Process> {
    let base_path = format!("{}/.igniter/procs", home_dir().unwrap().display());

    std::fs::read_dir(base_path).unwrap().map(|entry| {
        let file = entry.unwrap();
        
        read_process_file(format!("{}", file.path().to_str().unwrap()))
    }).collect()
}

fn kill_process(pid: i32) -> nix::Result<()> {
    nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), nix::sys::signal::SIGTERM)
}

fn launch_process(mut process: Process) -> std::result::Result<Child, String> {
    let mut command = Command::new(process.cmd.clone());
    let args = process.args.clone();

    for a in args {
        command.args(&a);
    }

    if let Ok(child) = command.spawn() {
        let child_pid = child.id() as i32;  
        let current_pid = i32::from(nix::unistd::getpid());

        process.set_pid(current_pid.clone());
        process.set_child_pid(child_pid.clone());
        save_process_file(format!("{}", process.pid), serde_json::to_string(&process).unwrap());

        println!("Started child process with PID: {}", child.id().clone());
        
        Ok(child)
    } else {
        Err(String::from("Child process not spawned"))
    }
}

fn register_sigterm_handler() {
    println!("Registering sigterm handler.");

    ctrlc::set_handler(move || {
        println!("SIGTERM arrived!");

        let current_pid = i32::from(nix::unistd::getpid());
        let process = read_process_file(build_process_filename(format!("{}", current_pid)));

        if let Ok(_) = kill_process(process.child_pid.clone()) {
            println!("Child closed");
            delete_process_file(process.pid.clone());
        } else {
            println!("error closing child");
        }
    }).expect("Error setting Ctrl-C handler");
}

fn get_current_process() -> i32 {
    i32::from(nix::unistd::getpid())
}

fn start_monitor(mut process: Process) {
    if let Ok(mut child) = launch_process(process.clone()) {
        if let Ok(status) = child.wait() {
            match status.code() {
                Some(code) => {
                    if code > 0 {
                        println!("Child process ended with errors, retry!");
                        process.increment_retries();

                        if process.retries <= process.max_retries {
                            start_monitor(process);
                        } else {
                            println!("Too many retries, stopping!");
                            delete_process_file(get_current_process());
                            exit(0);
                        }
                    } else {
                        delete_process_file(get_current_process());
                        println!("Child process ended with no errors.");
                        exit(0);
                    }
                },
                None => {
                    println!("Child process closed by signals. Stopping.");
                    delete_process_file(get_current_process());
                    exit(0);
            }
            }
        } else {
            println!("Process wasn not started");
        }
    }
}

fn monitor(data: &str) {
    let process: Process = serde_json::from_str(data).unwrap();
    register_sigterm_handler();
    start_monitor(process);
}

fn list() {
    let processes = get_active_processes();
    let mut table = Table::new();
    table.add_row(row!["PID", "NAME", "COMMAND", "ARGS", "RETRIES", "MAX RETRIES"]);
    
    for process in processes {
        table.add_row(Row::new(vec![
            Cell::new(format!("{}", process.pid).as_str()),
            Cell::new(process.name.as_str()),
            Cell::new(process.cmd.as_str()),
            Cell::new(format!("{:?}", process.args).as_str()),
            Cell::new(format!("{:?}", process.retries).as_str()),
            Cell::new(format!("{:?}", process.max_retries).as_str()),
        ]));
    }

    table.printstd();
}

fn stop(process: &str) {
    if let Some(searched_process) = get_active_processes().iter().find(|p| { p.name == String::from(process)}) {
        println!("Killing process with PID: {}", searched_process.pid.clone());
        kill_process(searched_process.pid).unwrap();
    } else {
        println!("No process to stop");
    }
}

fn read_settings() -> Settings {
    let mut config = config::Config::default();

    config
        .merge(config::File::new(".igniterc", FileFormat::Toml))
        .expect("No .igniterc file found!");

    config.try_into::<Settings>().unwrap()
}

fn main() {
    let matches = App::new(crate_name!())
        .version("0.1.0")
        .author(crate_authors!("\n"))
        .about("A simple process manager")
        .subcommand(SubCommand::with_name("monitor")
                .about("Monitors the provided command")
                .arg(Arg::with_name("command")
                    .help("the command to monitor")
                    .index(1)
                    .required(true)
                )
        )
        .subcommand(SubCommand::with_name("list")
                .about("list active process")
        )
        .subcommand(SubCommand::with_name("stop")
                .about("Stops an already running process given its name")
                .arg(Arg::with_name("process")
                    .help("The process to stop")
                    .index(1)
                    .required(true)
                )
        ).get_matches();

    match matches.subcommand() {
        ("monitor", Some(monitor_matches))  => monitor(monitor_matches.value_of("command").unwrap()),
        ("stop", Some(stop_matches))        => stop(stop_matches.value_of("process").unwrap()),
        ("list", Some(_))     => list(),
        ("", None)   => start_processes(), 
        _            => unreachable!(), 
    }
}