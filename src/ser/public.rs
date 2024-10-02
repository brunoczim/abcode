use std::{fmt, panic};

use serde::Serialize;
use thiserror::Error;
use tokio::{
    io::{self, AsyncWrite},
    sync::mpsc,
    task,
};

use super::internal::{BufferSink, ChannelBackend, ChannelSink, Serializer};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Internal writer disconnected")]
    Disconnected,
    #[error("Size {0} is too big for the protocol")]
    ExcessiveSize(usize),
    #[error("Size difference {0} is too big in magnitude for the protocol")]
    ExcessiveSizeDiff(isize),
    #[error("Skipping fields is not allowed")]
    SkipNotAllowed,
    #[error("I/O error writing to serialization target")]
    IO(
        #[from]
        #[source]
        io::Error,
    ),
    #[error("{0}")]
    Custom(String),
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Buffer limit {0} is too low")]
    BufLimitTooLow(usize),
}

#[derive(Debug, Clone)]
pub struct Config {
    batch_limit: usize,
    channel_limit: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self { batch_limit: 64, channel_limit: 64 }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_batch_limit(
        &mut self,
        byte_count: usize,
    ) -> Result<&mut Self, ConfigError> {
        if byte_count == 0 {
            Err(ConfigError::BufLimitTooLow(byte_count))?;
        }
        self.batch_limit = byte_count;
        Ok(self)
    }

    pub fn with_channel_limit(&mut self, byte_count: usize) -> &mut Self {
        self.channel_limit = byte_count;
        self
    }

    pub async fn serialize<T, W>(
        &self,
        device: W,
        value: T,
    ) -> Result<(), Error>
    where
        W: AsyncWrite + Unpin,
        T: Serialize + Send + 'static,
    {
        let (sender, receiver) = mpsc::channel(self.channel_limit);

        let backend = ChannelBackend::new(device, self.batch_limit, receiver);

        let mut serializer = Serializer::new(ChannelSink::new(sender));
        let block_handle =
            task::spawn_blocking(move || value.serialize(&mut serializer));

        backend.run().await?;
        match block_handle.await {
            Ok(actual_result) => actual_result?,
            Err(error) => panic::resume_unwind(error.into_panic()),
        }
        Ok(())
    }

    pub fn serialize_into_buffer<T>(&self, value: T) -> Result<Vec<u8>, Error>
    where
        T: Serialize,
    {
        let mut buffer = Vec::new();
        self.serialize_on_buffer(&mut buffer, value)?;
        Ok(buffer)
    }

    pub fn serialize_on_buffer<T>(
        &self,
        buffer: &mut Vec<u8>,
        value: T,
    ) -> Result<(), Error>
    where
        T: Serialize,
    {
        let mut serializer = Serializer::new(BufferSink::with_buffer(buffer));
        value.serialize(&mut serializer)
    }
}

pub async fn serialize<T, W>(device: W, value: T) -> Result<(), Error>
where
    W: AsyncWrite + Unpin,
    T: Serialize + Send + 'static,
{
    Config::default().serialize(device, value).await
}

pub fn serialize_into_buffer<T>(value: T) -> Result<Vec<u8>, Error>
where
    T: Serialize,
{
    Config::default().serialize_into_buffer(value)
}

pub fn serialize_on_buffer<T>(
    buffer: &mut Vec<u8>,
    value: T,
) -> Result<(), Error>
where
    T: Serialize,
{
    Config::default().serialize_on_buffer(buffer, value)
}
