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
use nix::Result;
use std::process::{Command};
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
    args: Vec<Vec<String>>
}

#[derive(Debug, Deserialize)]
struct Settings {
    process: Vec<Process>,
}

impl Process {
    fn set_pid(&mut self, pid: i32) {
        self.pid = pid;
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

fn delete_proces_file(name: String) {
    std::fs::remove_file(build_process_filename(name)).unwrap();
}

fn build_process_filename(name: String) -> String {
    let base_path = format!("{}/.igniter/procs", home_dir().unwrap().display());

    std::fs::create_dir_all(base_path.clone()).unwrap();
    format!("{}/{}.json",base_path, name)
}

fn read_process_file(path: &str) -> Process {
    let mut file = File::open(path).unwrap();
    let mut content = String::new();

    file.read_to_string(&mut content).unwrap();

    serde_json::from_str(content.as_str()).unwrap()
}

fn monitor(data: &str) {
    let mut process: Process = serde_json::from_str(data).unwrap();
    let mut command = Command::new(process.cmd.clone());
    let args = process.args.clone();

    for a in args {
        command.args(&a);
    }
    
    if let Ok(mut child) = command.spawn() {
        let child_pid = child.id() as i32;  
        let current_pid = i32::from(nix::unistd::getpid());

        process.set_pid(current_pid.clone());

        ctrlc::set_handler(move || {
            if let Ok(_) = kill_process(child_pid) {
                println!("Child closed");
                delete_proces_file(format!("{}", current_pid));
            } else {
                 println!("error closing child");
            }
        }).expect("Error setting Ctrl-C handler");

        save_process_file(format!("{}", process.pid), serde_json::to_string(&process).unwrap());

        child.wait().expect("Command did not start");
        println!("Command finished");
        delete_proces_file(format!("{}", process.pid));
    } else {
        println!("Could not start command.");
    }
}

fn get_active_processes() -> Vec<Process> {
    let base_path = format!("{}/.igniter/procs", home_dir().unwrap().display());

    std::fs::read_dir(base_path).unwrap().map(|entry| {
        let file = entry.unwrap();
        
        read_process_file(file.path().to_str().unwrap())
    }).collect()
} 

fn list() {
    let processes = get_active_processes();
    let mut table = Table::new();
    table.add_row(row!["PID", "NAME", "COMMAND", "ARGS", "STATUS"]);
    
    for process in processes {
        table.add_row(Row::new(vec![
            Cell::new(format!("{}", process.pid).as_str()),
            Cell::new(process.name.as_str()),
            Cell::new(process.cmd.as_str()),
            Cell::new(format!("{:?}", process.args).as_str()),
            Cell::new("??"),
        ]));
    }

    table.printstd();
}

fn kill_process(pid: i32) -> Result<()> {
    nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), nix::sys::signal::SIGTERM)
}

fn stop(process: &str) {
    if let Some(searched_process) = get_active_processes().iter().find(|p| { p.name == String::from(process)}) {
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