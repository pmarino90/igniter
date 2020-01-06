use std::io;
use std::process::Command;
use std::process::{Child, ExitStatus};

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
    pub fn status(&mut self) -> io::Result<JobStatus> {
        let child = &mut self.child;
        match child {
            None => Ok(JobStatus::NotStarted),
            Some(child) => match child.try_wait()? {
                Some(status) => Ok(JobStatus::Exited(status)),
                None => Ok(JobStatus::Running(child)),
            },
        }
    }

    pub fn required_action(
        &mut self,
        desired_status: &DesiredJobStatus,
    ) -> io::Result<Option<Action>> {
        let current_status = self.status()?;
        let result = match (current_status, desired_status) {
            (Status::NotStarted, DesiredJobStatus::NotStarted) => None,
            (Status::NotStarted, DesiredJobStatus::Running(_)) => Some(Action::Start),
            (Status::NotStarted, DesiredJobStatus::Exited(_)) => Some(Action::Start),

            (Status::Running(_), DesiredJobStatus::Running(_)) => None,
            (Status::Running(_), DesiredJobStatus::NotStarted) => Some(Action::Stop),
            (Status::Running(_), DesiredJobStatus::Exited(_)) => {
                // We'll wait
                None
            }

            (Status::Exited(_), Status::NotStarted) => None,
            (Status::Exited(_), Status::Exited(_)) => None,
            (Status::Exited(_), Status::Running(_)) => Some(Action::Start),
        };
        Ok(result)
    }

    pub fn try_reconciliate(&mut self, desired_status: &DesiredJobStatus) -> io::Result<()> {
        match self.required_action(desired_status)? {
            Some(Action::Stop) => self.stop(),
            Some(Action::Start) => self.start(),
            Some(Action::Restart) => self.stop().and_then(|_| self.start()),
            None => Ok(()),
        }
    }

    pub fn start(&mut self) -> io::Result<()> {
        println!("[{}] starting", self.name);
        self.command.spawn().and_then(|child| {
            self.child = Some(child);
            Ok(())
        })
    }

    pub fn stop(&mut self) -> io::Result<()> {
        match &mut self.child {
            Some(child) => child.kill(),
            None => Ok(()),
        }
    }
}

pub enum Status<T> {
    NotStarted,
    Running(T),
    Exited(ExitStatus),
}

pub type JobStatus<'a> = Status<&'a mut Child>;
pub type DesiredJobStatus = Status<()>;

pub enum Action {
    Start,
    Stop,
    Restart,
}
