use sail_core::{Message, Reply, Request, Response};
use std::{
    io::{self, BufRead, BufReader, Lines, Write},
    os::unix::net::UnixStream,
    path::Path,
};

pub struct Socket {
    reader: Lines<BufReader<UnixStream>>,
    writer: UnixStream,
    next_id: u8,
}

impl Socket {
    pub fn connect(socket_path: impl AsRef<Path>) -> Result<Self, SocketError> {
        let stream = UnixStream::connect(socket_path)?;

        Ok(Self {
            reader: BufReader::new(stream.try_clone()?).lines(),
            writer: stream,
            next_id: 1,
        })
    }

    pub fn send_request(&mut self, request: Request) -> Result<Response, SocketError> {
        let message = Message {
            id: self.next_id,
            request,
        };

        self.next_id += 1;

        self.writer
            .write_all(format!("{}\n", serde_json::to_string(&message)?).as_bytes())?;

        let reply: Reply = serde_json::from_str(&self.reader.next().ok_or(SocketError::NoReply)??)?;

        if reply.regarding != message.id {
            return Err(SocketError::ReplyMismatch);
        }

        Ok(reply.response)
    }
}

#[derive(Debug)]
pub enum SocketError {
    Io(io::Error),
    NoReply,
    Serialization(serde_json::Error),
    ReplyMismatch,
}

impl From<io::Error> for SocketError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for SocketError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value)
    }
}
