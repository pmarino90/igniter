//! This module handles the communication between the Igniter manager
//! and clients.
//!
//! The RPC provide one type for each end of the communication:
//!
//!   - `Server` - Can check for pending requests with the
//!                non-blocking method `try_receive`.
//!
//!   - `Client` - It can use `.request()` to make a request to the
//!                corresponding server.
//!
//! Both ends can exchange values of type `Message`. Messages are
//! serialized/deserialized with Serde and sent through Unix Sockets.

use std::fs;
use std::io;
use std::io::prelude::*;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json;

use crate::config;

#[derive(Serialize, Deserialize, Debug)]
/// A message to be exchanged between a `Client` and `Server`.
pub enum Message {
    /// Start a new process with the given config name and config.
    Start(String, config::Process),
    /// Stop a process with a given name.
    Stop(String),
    Quit,
    Status,
}

fn encode(msg: &Message) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(msg)
}

fn decode(str: &[u8]) -> Result<Message, serde_json::Error> {
    serde_json::from_slice(str)
}

pub fn get_socket_file(config_dir: &Path) -> PathBuf {
    config_dir.join("daemon.socket")
}

pub struct Client {
    stream: UnixStream,
}

impl Client {
    /// Create a client based on the `config_dir`.
    pub fn new(config_dir: &Path) -> io::Result<Client> {
        let socket_file = get_socket_file(config_dir);
        let stream = UnixStream::connect(socket_file)?;
        Ok(Client { stream })
    }

    /// Send Message to the corresponding server.
    ///
    /// Fail if the request can't be served, for example, because the
    /// server is not running.
    pub fn request(&mut self, msg: &Message) -> io::Result<()> {
        let bytes = &encode(msg).unwrap();
        self.stream.write_all(bytes)?;
        Ok(())
    }
}

pub struct Server {
    listener: UnixListener,
}

impl Server {
    /// Create and bind a new server based on `config_dir.`
    pub fn new(config_dir: &Path) -> io::Result<Server> {
        let socket_file = get_socket_file(config_dir);
        let _ = fs::remove_file(&socket_file);
        let listener = UnixListener::bind(&socket_file).unwrap();
        listener
            .set_nonblocking(true)
            .expect("can't listen on unix socket");
        Ok(Server { listener })
    }

    /// Try to receive a message.
    ///
    /// If no message is available, A Ok(None) is returned. Errors are
    /// only returned when we can't read messages from the socket for
    /// some reason.
    pub fn try_receive(&mut self) -> io::Result<Option<Message>> {
        match self.listener.accept() {
            Ok((mut stream, _)) => {
                let mut request: Vec<u8> = Vec::new();
                stream.read_to_end(&mut request)?;
                let msg = decode(&request)?;
                println!("msg: {:?}", msg);
                Ok(Some(msg))
            }
            Err(err) => match err.kind() {
                io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => Ok(None),
                _ => Err(err),
            },
        }
    }
}
