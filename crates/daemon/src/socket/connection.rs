use sail_core::socket::{SocketMessage, SocketReply};
use tokio::{
    io::{self, AsyncWriteExt, BufReader, Lines},
    net::unix::{OwnedReadHalf, OwnedWriteHalf, SocketAddr},
};

#[derive(Debug)]
pub struct SocketConnection {
    reader: Lines<BufReader<OwnedReadHalf>>,
    writer: OwnedWriteHalf,
    pub address: SocketAddr,
}

impl SocketConnection {
    pub fn new(
        reader: Lines<BufReader<OwnedReadHalf>>,
        writer: OwnedWriteHalf,
        address: SocketAddr,
    ) -> Self {
        Self {
            reader,
            writer,
            address,
        }
    }

    pub async fn accept(&mut self) -> Result<Option<SocketMessage>, ConnectionError> {
        let maybe_line = self
            .reader
            .next_line()
            .await
            .map_err(ConnectionError::Read)?;

        Ok(match maybe_line {
            Some(line) => Some(
                serde_json::from_str::<SocketMessage>(&line)
                    .map_err(ConnectionError::Deserialization)?,
            ),
            None => None,
        })
    }

    pub async fn reply(&mut self, reply: SocketReply) -> Result<(), ConnectionError> {
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
