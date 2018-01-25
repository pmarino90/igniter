use monitor::{Process, ProcessData};
use config::{FileFormat, Config, File};

#[derive(Debug, Deserialize)]
pub struct Settings {
  pub process: Vec<ProcessData>,
}

impl Settings {
  pub fn read() -> Settings {
    let mut config = Config::default();

    config
        .merge(File::new(".igniterc", FileFormat::Toml))
        .expect("No .igniterc file found!");

    config.try_into::<Settings>().unwrap()
  }

  pub fn list_procs(&self) -> Vec<Process> {
    let procs = self.process.clone();
    
    procs.into_iter().map(|p| { Process{ data:  p } }).collect()
  }
}