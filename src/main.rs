#[macro_use]
extern crate clap;
#[macro_use]
extern crate prettytable;

extern crate ctrlc;
extern crate igniter;
extern crate dirs;

use igniter::os;
use igniter::monitor;
use igniter::monitor::Process;
use igniter::settings::Settings;

use clap::{App, Arg, SubCommand};
use std::process::Command;
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;

fn start_processes() {
    println!("Starting processes found in .igniterc");
    let settings = Settings::read();
    let processes = settings.list_procs();

    for p in processes {
        let data = p.serialize().unwrap();

        println!("Name: {}", p.data.name);
        println!("Command: {}", p.data.cmd);

        let child = Command::new(std::env::current_exe().unwrap())
            .args(&[String::from("monitor"), data.clone()])
            .spawn()
            .expect("Error while starting command");

        println!("Manager process started with PID: {}", child.id());
    }
}

fn register_sigterm_handler(name: String) {
    println!("Registering sigterm handler.");

    ctrlc::set_handler(move || {
        println!("SIGTERM arrived!");

        let process = monitor::file::read(monitor::file::path_from_name(name.clone())).unwrap();

        if let Ok(_) = os::kill(process.data.child_pid) {
            println!("Child closed");
        } else {
            println!("error closing child");
        }
    }).expect("Error setting Ctrl-C handler");
}

fn monitor(data: &str) {
    let mut process = Process::from(data);

    register_sigterm_handler(process.data.name.clone());
    monitor::start(&mut process);
}

fn list(all: bool) {
    let processes = monitor::list_processes(all);
    let mut table = Table::new();
    table.add_row(row![
        "MONITOR PID",
        "CHILD PID",
        "NAME",
        "COMMAND",
        "ARGS",
        "RETRIES",
        "MAX RETRIES"
    ]);

    for process in processes {
        table.add_row(Row::new(vec![
            Cell::new(format!("{}", process.data.monitor_pid).as_str()),
            Cell::new(format!("{}", process.data.child_pid).as_str()),
            Cell::new(process.data.name.as_str()),
            Cell::new(process.data.cmd.as_str()),
            Cell::new(format!("{:?}", process.data.args).as_str()),
            Cell::new(format!("{:?}", process.data.retries).as_str()),
            Cell::new(format!("{:?}", process.data.max_retries).as_str()),
        ]));
    }

    table.printstd();
}

fn stop(process_name: &str) {
    if let Some(searched_process) = monitor::list_processes(false)
        .iter()
        .find(|p| p.data.name == String::from(process_name))
    {
        println!(
            "Killing process {} with Monitor PID: {}",
            process_name,
            searched_process.data.monitor_pid.clone()
        );
        searched_process.kill().unwrap();
    } else {
        println!("Process {} not found", process_name);
    }
}

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about("A simple process manager")
        .subcommand(
            SubCommand::with_name("monitor")
                .about("[INTERNAL] Monitors the provided command data as JSON")
                .arg(
                    Arg::with_name("data")
                        .help("Data needed to start the monitoring process.")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("list active processes")
                .arg(
                    Arg::with_name("all")
                        .long("all")
                        .short("-a")
                        .help("Show all processes"),
                ),
        )
        .subcommand(
            SubCommand::with_name("stop")
                .about("Stops an already running process given its name")
                .arg(
                    Arg::with_name("process")
                        .help("The process to stop")
                        .index(1)
                        .required(true),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("monitor", Some(monitor_matches)) => monitor(monitor_matches.value_of("data").unwrap()),
        ("stop", Some(stop_matches)) => stop(stop_matches.value_of("process").unwrap()),
        ("list", Some(list_matches)) => list(list_matches.is_present("all")),
        ("", None) => start_processes(),
        _ => unreachable!(),
    }
}
