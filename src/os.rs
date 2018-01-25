pub use nix::Result;

use nix::sys::signal;
use nix::unistd::{Pid, getpid};

pub fn kill(pid: i32) -> Result<()> {
  signal::kill(Pid::from_raw(pid), signal::SIGTERM)
}

pub fn current_pid() -> i32 {
  i32::from(getpid())
}