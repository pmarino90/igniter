pub mod file;
mod process;

pub use self::process::ProcessData;
pub use self::process::Process;

use std::fs;

pub fn start(process: &mut Process) {
  println!("Start monitoring");
  if let Ok(mut child) = process.spawn() {
    if let Ok(status) = child.wait() {
      match status.code() {
        Some(code) => {
          if code > 0 {
            println!("Child process ended with errors, retry!");
            process.increment_retries();
            
            if process.should_retry() {
              start(process);
            } else {
              println!("Too many retries, stopping!");
            }
          } else {
            println!("Child process ended with no errors.");
          }
        },
        None => {
          println!("Child process closed by signals. Stopping.");
        }
      }
    } else {
      println!("Process was not started");
    }
  }
}

pub fn list_processes(all: bool) -> Vec<Process> {
  let base_path = file::procs_path();
  
  fs::read_dir(base_path).unwrap().map(|entry| {
    let file = entry.unwrap();
    let path = format!("{}", file.path().to_str().unwrap());
    
    file::read(path).unwrap()
  }).filter(|p| { all || p.is_active() }).collect()
}