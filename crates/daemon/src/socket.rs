mod connection;

pub use connection::SocketConnection;

use std::{
    env::{self, VarError},
    os::fd::FromRawFd,
};

use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    net::UnixListener,
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

        Ok(SocketConnection::new(
            BufReader::new(reader).lines(),
            writer,
            socket_address,
        ))
    }
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
