//! A temporary wrapper around concurrently.

use std::env;
use std::env;
use std::io;

use clap::{App, Arg};

use std::process;

const PKG_NAME: &'static str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const PKG_AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const PKG_DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
const PKG_GIT_COMMIT: Option<&'static str> = option_env!("GIT_COMMIT");

fn main() -> io::Result<()> {
    let _matches = App::new(PKG_NAME)
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
        .arg(Arg::with_name("kill-others-on-fail").long("kill-others-on-fail"))
        .arg(Arg::with_name("raw").long("raw"))
        .get_matches();

    let mut PATH = env::var("PATH").unwrap_or("");

    process::Command::new("yarn")
        .arg("run")
        // .env("PATH", "/bin")
        .status()
        .expect("fail to run yarn");

    Ok(())
}
