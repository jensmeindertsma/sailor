use sailor_core::control::{Message, Reply, Request, Response};
use std::{
    io::{self, BufRead, BufReader, Lines, Write},
    os::unix::net::UnixStream,
    path::Path,
};

pub struct Controller {
    reader: Lines<BufReader<UnixStream>>,
    writer: UnixStream,
}

impl Controller {
    pub fn connect(socket_path: impl AsRef<Path>) -> Result<Self, Error> {
        let stream =
            UnixStream::connect(socket_path).map_err(|e| Error::ConnectionFailure(e.kind()))?;

        Ok(Self {
            reader: BufReader::new(
                stream
                    .try_clone()
                    .map_err(|e| Error::ConnectionFailure(e.kind()))?,
            )
            .lines(),
            writer: stream,
        })
    }

    pub fn request(&mut self, request: Request) -> Response {
        let message = Message {
            id: rand::random(),
            request,
        };

        self.writer
            .write_all(
                format!(
                    "{}\n",
                    serde_json::to_string(&message).expect("message serialization should succeed")
                )
                .as_bytes(),
            )
            .expect("writing to the stream should succeed");

        let reply: Reply = serde_json::from_str(
            &self
                .reader
                .next()
                .expect("reading from the stream should succeed")
                .expect("there should be a response"),
        )
        .expect("reply deserialization should succeed");

        if reply.regarding != message.id {
            panic!("Reply ID did not match!")
        }

        reply.response
    }
}

#[derive(Debug)]
pub enum Error {
    ConnectionFailure(io::ErrorKind),
}
