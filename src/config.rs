use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use serde::Deserialize;
use toml;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub job: HashMap<String, Job>,
}

#[derive(Deserialize, Debug)]
pub struct Job {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub enum Error {
    TomlError(toml::de::Error),
    IOError(io::Error),
}

pub fn load_config<P: AsRef<Path>>(file: P) -> Result<Config, Error> {
    let content = fs::read_to_string(file).map_err(Error::IOError)?;
    let config: Config = toml::from_str(&content).map_err(Error::TomlError)?;
    Ok(config)
}
