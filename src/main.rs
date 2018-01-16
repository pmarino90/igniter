extern crate config;
extern crate serde;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;

use clap::{Arg, App, SubCommand};
use std::process::{Command, Stdio};
use config::{FileFormat};

#[derive(Debug, Deserialize)]
struct Process {
    name: String,
    cmd: String,
}

#[derive(Debug, Deserialize)]
struct Settings {
    process: Vec<Process>,
}

fn start_processes(processes: Vec<Process>) {
    println!("Starting processes found in .igniterc");

    for p in processes {
        println!("Name: {}", p.name);
        println!("Command: {}", p.cmd);
        println!("");

        let child = Command::new("igniter")
        .args(&[String::from("monitor"), p.cmd])
        .stdout(Stdio::null())
        .spawn()
        .expect("Error while starting command");

        println!("Process started with PID: {}", child.id());
    }
}

fn monitor(cmd: &str) {
    let mut command = Command::new(cmd);

    if let Ok(mut child) = command.spawn() {        
        child.wait().expect("Command did not start");
        println!("Command finished");
    } else {
        println!("Could not start command");
    }
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
        ).get_matches();


    match matches.subcommand() {
        ("monitor", Some(clone_matches)) => monitor(clone_matches.value_of("command").unwrap()),
        ("", None)   => start_processes(settings.process), 
        _            => unreachable!(), 
    }
}