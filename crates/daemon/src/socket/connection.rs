use sail_core::{Message, Reply};
use tokio::{
    io::{self, AsyncWriteExt, BufReader, Lines},
    net::unix::{OwnedReadHalf, OwnedWriteHalf, SocketAddr},
};

#[derive(Debug)]
pub struct SocketConnection {
    reader: Lines<BufReader<OwnedReadHalf>>,
    writer: OwnedWriteHalf,
    pub socket_address: SocketAddr,
}

impl SocketConnection {
    pub fn new(
        reader: Lines<BufReader<OwnedReadHalf>>,
        writer: OwnedWriteHalf,
        socket_address: SocketAddr,
    ) -> Self {
        Self {
            reader,
            writer,
            socket_address,
        }
    }

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
