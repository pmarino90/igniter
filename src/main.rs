use std::env;
use std::fs;
use std::io;

use clap::{App, AppSettings, Arg, SubCommand};

use igniter::config;
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

const PROCESS_NAME_ARG: &str = "PROCESS_NAME";
const NO_DAEMON_ARG: &str = "no-daemon";

fn main() -> io::Result<()> {
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
                Arg::with_name(NO_DAEMON_ARG)
                    .long("no-daemon")
                    .help("Do not dettach the server from the current terminal."),
            ),
        )
        .subcommand(SubCommand::with_name("status"))
        .subcommand(SubCommand::with_name("kill"))
        .subcommand(SubCommand::with_name("start").arg(Arg::with_name(PROCESS_NAME_ARG).index(1)))
        .subcommand(SubCommand::with_name("stop").arg(Arg::with_name(PROCESS_NAME_ARG).index(1)))
        .get_matches();

    let config_dir = create_config_dir().expect("couldn't create config dir.");

    let current_dir = env::current_dir()?;

    match matches.subcommand() {
        ("server", Some(submatches)) => {
            let daemonize = !submatches.is_present("no-daemon");
            manager::server::start(&config_dir, daemonize);
        }

        ("kill", _) => {
            let mut client = rpc::Client::new(&config_dir).unwrap();
            client.request(&rpc::Message::Quit).unwrap();
        }

        ("status", _) => {
            let mut client = rpc::Client::new(&config_dir).unwrap();
            client.request(&rpc::Message::Status).unwrap();
        }

        ("start", Some(submatches)) => {
            let mut client = rpc::Client::new(&config_dir).unwrap();

            let config = config::load_config(current_dir.join("igniter.toml"))
                .expect("could not load configuration");

            let process_name = submatches.value_of(PROCESS_NAME_ARG);

            for (name, process) in config.process {
                if process_name.unwrap_or(&name) == name {
                    client.request(&rpc::Message::Start(name, process)).unwrap();
                }
            }
        }

        ("stop", Some(_submatches)) => {}

        _ => unreachable!(),
    }

    Ok(())
}
