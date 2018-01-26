use monitor::file;
use os;

use std::io;
use std::io::Error;
use std::process::{Command, Child};
use serde_json;

type CmdArgs = Vec<Vec<String>>;
type CmdEnv = Vec<Vec<String>>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProcessData {
    pub name: String,
    pub cmd: String,
    #[serde(default)]
    pub monitor_pid: i32,
    #[serde(default)]
    pub child_pid: i32,
    #[serde(default)]
    pub args: CmdArgs,
    #[serde(default)]
    pub env: CmdEnv,
    #[serde(default)]
    pub retries: i32,
    #[serde(default)]
    pub max_retries: i32,
}

pub struct Process {
  pub data: ProcessData,
}

impl Process {
  pub fn spawn(&mut self) -> Result<Child, Error> {
    let mut command = Command::new(self.data.cmd.clone());
    let args = self.data.args.clone();
    let env =  self.data.env.clone();

    build_args(&mut command, args);
    build_env_vars(&mut command, env);

    match command.spawn() {
      Ok(child) => {
        let child_pid = child.id() as i32;  
        let current_pid = os::current_pid();

        self.monitor_pid(current_pid.clone());
        self.child_pid(child_pid.clone());
        self.save_state()?;

        println!("Started child process with PID: {}", child.id().clone());
        
        Ok(child)
      },
      Err(err) => Err(err),
    } 
  }

  pub fn kill(&self) -> os::Result<()> {
    os::kill(self.data.monitor_pid)
  }

  pub fn monitor_pid(&mut self, pid: i32) {
    self.data.monitor_pid = pid;
  }

  pub fn child_pid(&mut self, pid: i32) {
    self.data.child_pid = pid;
  }

  pub fn increment_retries(&mut self) {
    self.data.retries = self.data.retries + 1;
  }

  pub fn should_retry(&self) -> bool {
    self.data.retries <= self.data.max_retries
  }

  pub fn is_active(&self) -> bool {
    match os::ps(self.data.monitor_pid) {
      Some(_) => true,
      None => false
    }
  }

  pub fn save_state(&self) -> io::Result<()> {
    println!("saving state");
    file::save(file::path_from_name(self.data.name.clone()), &self)
  }

  pub fn serialize(&self) -> serde_json::Result<String> {
    serde_json::to_string(&self.data.clone())
  }
}

impl From<String> for Process {
  fn from(data: String) -> Process {
     Process::from(data.as_str())
  }
}

impl<'a> From<&'a str> for Process {
  fn from(data: &'a str) -> Process {
    let process_data: ProcessData = serde_json::from_str(&data).unwrap();

    Process{
      data: process_data,
    }
  }
}

fn build_args(cmd: &mut Command, args: CmdArgs) {
  for a in args {
    cmd.args(&a);
  }
}

fn build_env_vars(cmd: &mut Command, vars: CmdEnv) {
  println!("{:?}", vars);

  for v in vars {
    let key = v[0].clone();
    let value = v[1].clone();

    cmd.env(key, value);
  }
}