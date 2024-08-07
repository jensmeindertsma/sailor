use std::{
    env::{self, VarError},
    io,
    os::fd::FromRawFd,
};

use sail_core::{Message, Reply};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    net::{
        unix::{OwnedReadHalf, OwnedWriteHalf, SocketAddr},
        UnixListener,
    },
};

pub struct Socket {
    listener: UnixListener,
}

impl Socket {
    pub fn attach() -> Result<Self, SocketError> {
        use std::os::unix::net::UnixListener as StdUnixListener;

        let fd = env::var("LISTEN_FDS").map_err(SocketError::InvalidVariable)?;

        let fd: i32 = fd.parse().map_err(|_| VarError::NotUnicode(fd.into()))?;

        if fd != 1 {
            return Err(SocketError::UnexpectedFileDescriptor(fd));
        }

        // SAFETY: this file descriptor comes from systemd
        let std_listener = unsafe { StdUnixListener::from_raw_fd(3) };

        std_listener.set_nonblocking(true)?;

        Ok(Self {
            listener: UnixListener::from_std(std_listener)?,
        })
    }

    pub async fn accept(&self) -> Result<SocketConnection, io::Error> {
        let (stream, socket_address) = self.listener.accept().await?;
        let (reader, writer) = stream.into_split();

        Ok(SocketConnection {
            reader: BufReader::new(reader).lines(),
            writer,
            socket_address,
        })
    }
}

#[derive(Debug)]
pub struct SocketConnection {
    reader: Lines<BufReader<OwnedReadHalf>>,
    writer: OwnedWriteHalf,
    pub socket_address: SocketAddr,
}

impl SocketConnection {
    pub async fn accept(&mut self) -> Result<Option<Message>, ConnectionError> {
        let maybe_line = self
            .reader
            .next_line()
            .await
            .map_err(ConnectionError::Read)?;

        Ok(match maybe_line {
            Some(line) => Some(
                serde_json::from_str::<Message>(&line).map_err(ConnectionError::Deserialization)?,
            ),
            None => None,
        })
    }

    pub async fn reply(&mut self, reply: Reply) -> Result<(), ConnectionError> {
        self.writer
            .write_all(
                format!(
                    "{}\n",
                    serde_json::to_string(&reply).map_err(ConnectionError::Serialization)?
                )
                .as_bytes(),
            )
            .await
            .map_err(ConnectionError::Write)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum ConnectionError {
    Deserialization(serde_json::Error),
    Read(io::Error),
    Serialization(serde_json::Error),
    Write(io::Error),
}

#[derive(Debug)]
pub enum SocketError {
    Io(io::Error),
    UnexpectedFileDescriptor(i32),
    InvalidVariable(VarError),
}

impl From<io::Error> for SocketError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<VarError> for SocketError {
    fn from(value: VarError) -> Self {
        Self::InvalidVariable(value)
    }
}
