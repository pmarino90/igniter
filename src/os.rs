pub use nix::Result;

use nix::sys::signal;
use nix::unistd::{Pid, getpid};
use std::process::Command;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Process {
  pid: i32,
  tty: String,
  time: String,
  cmd: String,
}

pub fn kill(pid: i32) -> Result<()> {
  signal::kill(Pid::from_raw(pid), signal::SIGTERM)
}

pub fn current_pid() -> i32 {
  i32::from(getpid())
}

pub fn ps(pid: i32) -> Option<Process> {
  let mut ps = Command::new("ps");

  ps.arg("-p").arg(format!("{}", pid).as_str());

  match ps.output() {
    Ok(out) => parse_ps_out(format!("{}", String::from_utf8_lossy(&out.stdout))),
    Err(_) => None,
  }
}

fn parse_ps_out(out: String) -> Option<Process> {
  let mut lines: Vec<&str> = out.split("\n").collect();
  
  lines.remove(0);
  lines.pop();
  let procs: Vec<Process> = lines.iter().map(|p| { 
    let fields: Vec<&str> = p.split(" ").collect();

    Process {
      pid: FromStr::from_str(fields[0]).unwrap(),
      tty: format!("{}", fields[1]),
      time: format!("{}", fields[2]),
      cmd: format!("{}", fields[3]),
    }
  }).collect();

  if procs.len() > 0 { Some(procs[0].clone()) } else { None } 
}