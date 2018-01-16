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

fn start_processes(processes: Vec<Process>) {
    println!("Starting processes found in .igniterc");

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
        process.set_pid(i32::from(nix::unistd::getpid()));

        ctrlc::set_handler(move || {
            if let Ok(_) = nix::sys::signal::kill(nix::unistd::Pid::from_raw(child_pid), nix::sys::signal::SIGTERM) {
                println!("Child closed");
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

fn list() {
    let base_path = format!("{}/.igniter/procs", home_dir().unwrap().display());
    let mut table = Table::new();

    table.add_row(row!["PID", "NAME", "COMMAND", "ARGS", "STATUS"]);

    for e in std::fs::read_dir(base_path).unwrap() {
        let file = e.unwrap();
        let process  = read_process_file(file.path().to_str().unwrap());

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

fn main() {
    let mut config = config::Config::default();

    config
        .merge(config::File::new(".igniterc", FileFormat::Toml))
        .expect("No .igniterc file found!");

    let settings: Settings = config.try_into::<Settings>().unwrap();
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
        ).get_matches();


    match matches.subcommand() {
        ("monitor", Some(monitor_matches))  => monitor(monitor_matches.value_of("command").unwrap()),
        ("list", Some(_))     => list(),
        ("", None)   => start_processes(settings.process), 
        _            => unreachable!(), 
    }
}