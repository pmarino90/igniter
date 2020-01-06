use std::fs;
use std::io;
use std::io::prelude::*;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
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
    pub fn new(config_dir: &Path) -> io::Result<Client> {
        let socket_file = get_socket_file(config_dir);
        let stream = UnixStream::connect(socket_file)?;
        Ok(Client { stream })
    }

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
    pub fn new(config_dir: &Path) -> io::Result<Server> {
        let socket_file = get_socket_file(config_dir);
        let _ = fs::remove_file(&socket_file);
        let listener = UnixListener::bind(&socket_file).unwrap();
        listener
            .set_nonblocking(true)
            .expect("can't listen on unix socket");
        Ok(Server { listener })
    }

    pub fn try_receive(&mut self) -> io::Result<Option<Message>> {
        match self.listener.accept() {
            Ok((mut stream, _)) => {
                let mut request: Vec<u8> = Vec::new();
                stream.read_to_end(&mut request).expect("couldn't command");

                let msg = decode(&request).unwrap();

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
