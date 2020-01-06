use std::fs;
use std::io;

use clap::{App, AppSettings, Arg, SubCommand};

use igniter::manager;
use igniter::rpc;

const PKG_NAME: &'static str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const PKG_AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const PKG_DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
const PKG_GIT_COMMIT: Option<&'static str> = option_env!("GIT_COMMIT");

fn create_config_dir() -> Option<std::path::PathBuf> {
    let mut config_dir = dirs::home_dir().expect("can't find home directory");
    config_dir.push(".igniter/");
    fs::create_dir(&config_dir)
        .or_else(|err| match err.kind() {
            io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(err),
        })
        .expect("can't create config directory");

    Some(config_dir)
}

fn main() {
    let matches = App::new(PKG_NAME)
        .version(
            format!(
                "{} (git: {})",
                PKG_VERSION,
                PKG_GIT_COMMIT.map(|c| &c[0..8]).unwrap_or("unknown")
            )
            .as_ref(),
        )
        .author(PKG_AUTHORS)
        .about(PKG_DESCRIPTION)
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("server").arg(
                Arg::with_name("no-daemon")
                    .long("no-daemon")
                    .help("Do not dettach the server from the current terminal."),
            ),
        )
        .subcommand(SubCommand::with_name("status"))
        .subcommand(SubCommand::with_name("kill"))
        .get_matches();

    let config_dir = create_config_dir().expect("couldn't create config dir.");

    if let Some(server_matches) = matches.subcommand_matches("server") {
        let daemonize = !server_matches.is_present("no-daemon");
        manager::server::start(&config_dir, daemonize);
    } else if let Some(_) = matches.subcommand_matches("kill") {
        let mut client = rpc::Client::new(&config_dir).unwrap();
        client.request(&rpc::Message::Quit).unwrap();
        println!("message sent!");
    } else if let Some(_) = matches.subcommand_matches("status") {
        let mut client = rpc::Client::new(&config_dir).unwrap();
        client.request(&rpc::Message::Status).unwrap();
        println!("running.");
    }
}
