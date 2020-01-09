use std::io;
use std::process::{Child, Command};

pub struct Process {
    pub name: String,
    command: Command,
    child: Option<Child>,
    pub desired_status: Status,
}

impl Process {
    pub fn new(name: String, program: String, args: Vec<String>) -> Process {
        let mut command = Command::new(program);
        command.args(args);
        Process {
            name,
            command,
            child: None,
            desired_status: Status::NotRunning,
        }
    }

    /// Return the current status of a process.
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

    pub fn required_action(&mut self) -> io::Result<Option<Action>> {
        let current_status = self.status()?;
        let result = match (current_status, self.desired_status) {
            (Status::NotRunning, Status::NotRunning) => None,
            (Status::NotRunning, Status::Running) => Some(Action::Start),
            (Status::Running, Status::Running) => None,
            (Status::Running, Status::NotRunning) => Some(Action::Stop),
        };
        Ok(result)
    }

    pub fn try_reconciliate(&mut self) -> io::Result<()> {
        match self.required_action()? {
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
        println!("[{}] stopping", self.name);
        match &mut self.child {
            Some(child) => {
                child.kill()?;
                Ok(())
            }
            None => Ok(()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Status {
    NotRunning,
    Running,
}

pub enum Action {
    Start,
    Stop,
}
