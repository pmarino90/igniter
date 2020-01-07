use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub processes: ConfigProcesses,
}

#[derive(Deserialize)]
pub struct ConfigProcesses {
    pub name: String,
    pub args: Vec<String>,
}

pub fn load_config() -> Result<Config> {}
