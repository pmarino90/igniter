use std::io;
use std::process::{Child, Command};

pub struct Job {
    name: String,
    command: Command,
    child: Option<Child>,
}

impl Job {
    pub fn new(name: String, program: String, args: Vec<String>) -> Job {
        let mut command = Command::new(program);
        command.args(args);
        Job {
            name,
            command,
            child: None,
        }
    }

    /// Return the current status of a job.
    pub fn status(&mut self) -> io::Result<Status> {
        let child = &mut self.child;
        match child {
            None => Ok(Status::NotRunning),
            Some(child) => match child.try_wait()? {
                Some(_) => Ok(Status::NotRunning),
                None => Ok(Status::Running),
            },
        }
    }

    pub fn required_action(&mut self, desired_status: &Status) -> io::Result<Option<Action>> {
        let current_status = self.status()?;
        let result = match (current_status, desired_status) {
            (Status::NotRunning, Status::NotRunning) => None,
            (Status::NotRunning, Status::Running) => Some(Action::Start),
            (Status::Running, Status::Running) => None,
            (Status::Running, Status::NotRunning) => Some(Action::Stop),
        };
        Ok(result)
    }

    pub fn try_reconciliate(&mut self, desired_status: &Status) -> io::Result<()> {
        match self.required_action(desired_status)? {
            Some(Action::Stop) => self.stop(),
            Some(Action::Start) => self.start(),
            None => Ok(()),
        }
    }

    fn start(&mut self) -> io::Result<()> {
        println!("[{}] starting", self.name);
        self.command.spawn().and_then(|child| {
            self.child = Some(child);
            Ok(())
        })
    }

    fn stop(&mut self) -> io::Result<()> {
        match &mut self.child {
            Some(child) => child.kill(),
            None => Ok(()),
        }
    }
}

pub enum Status {
    NotRunning,
    Running,
}

pub enum Action {
    Start,
    Stop,
}
