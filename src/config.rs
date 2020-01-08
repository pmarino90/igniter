//! Igniter configuration
//!

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use toml;

/// Top level configuration.
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub process: HashMap<String, Process>,
}

/// Configuration of an individual process.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Process {
    /// Program to execute to start the process.
    pub program: String,
    /// An array of arguments to pass to the program.
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug)]
pub enum Error {
    TomlError(toml::de::Error),
    IOError(io::Error),
}

/// Load a configuration file.
pub fn load_config<P: AsRef<Path>>(file: P) -> Result<Config, Error> {
    let content = fs::read_to_string(file).map_err(Error::IOError)?;
    let config: Config = toml::from_str(&content).map_err(Error::TomlError)?;
    Ok(config)
}
