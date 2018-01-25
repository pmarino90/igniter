use monitor::{Process};

use std::io;
use std::env::home_dir;
use std::fs;
use std::io::prelude::*;
use serde_json;

pub fn save(path: String, process: &Process) -> io::Result<()> {
  let serialized_data = serde_json::to_string(&process.data)?;
  
  fs::create_dir_all(procs_path())?;
  println!("saving file {:?}", path);
  match fs::File::create(path) {
    Ok(mut file) => {
      match file.write_all(serialized_data.as_bytes()) {
        Ok(_) => Ok(()),
        Err(err) => Err(err)
      }
    },
    Err(err) => Err(err)
  }
}

pub fn read(path: String) -> io::Result<Process> {
  match fs::File::open(path) {
    Ok(mut file) =>  {
      let mut content = String::new();
        
      file.read_to_string(&mut content)?;

      Ok(Process::from(content))
    },
    Err(err) => Err(err),
  }
}

pub fn delete(path: String) -> io::Result<()> {
  fs::remove_file(path)
}

pub fn path_from_pid(pid: i32) -> String {
    let base_path = procs_path();

    format!("{}/{}.json",base_path, pid)
}

pub fn path_from_name(process_name: String) -> String {
    let base_path = procs_path();

    format!("{}/{}.json",base_path, process_name)
}

pub fn procs_path() -> String {
  format!("{}/.igniter/procs", home_dir().unwrap().display())
}
