use std::{fmt, panic, string::FromUtf8Error};

use serde::Deserialize;
use thiserror::Error;
use tokio::{
    io::{self, AsyncRead},
    sync::mpsc,
    task,
};

use super::internal::{
    BufferSource,
    ChannelBackend,
    ChannelSource,
    Deserializer,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Any deserialization is not supported")]
    UnsupportedAny,
    #[error("Reader reached end of input too early")]
    PrematureEof,
    #[error("Reader expected end of input, found {0}")]
    ExpectedEof(u8),
    #[error("Deserializer disconnected losing bytes")]
    Disconnected,
    #[error("Size {0} is too big for this machine")]
    ExcessiveSize(u64),
    #[error("Size difference {0} is too big in magnitude for this machine")]
    ExcessiveSizeDiff(i64),
    #[error("Codepoint {0} is invalid")]
    InvalidCodePoint(u32),
    #[error(transparent)]
    Utf8(#[from] FromUtf8Error),
    #[error("I/O error reading from deserialization source")]
    IO(
        #[from]
        #[source]
        io::Error,
    ),
    #[error("{0}")]
    Custom(String),
}

impl serde::de::Error for Error {
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
    hard_eof: bool,
    request_channel_limit: usize,
    response_channel_limit: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hard_eof: false,
            request_channel_limit: 1,
            response_channel_limit: 1,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_hard_eof(&mut self) -> &mut Self {
        self.hard_eof = true;
        self
    }

    pub fn with_request_channel_limit(&mut self, limit: usize) -> &mut Self {
        self.request_channel_limit = limit;
        self
    }

    pub fn with_response_channel_limit(&mut self, limit: usize) -> &mut Self {
        self.response_channel_limit = limit;
        self
    }

    pub async fn deserialize<'de, T, R>(&self, device: R) -> Result<T, Error>
    where
        R: AsyncRead + Unpin,
        T: Deserialize<'de> + Send + 'static,
    {
        let (request_sender, request_receiver) =
            mpsc::channel(self.request_channel_limit);
        let (response_sender, response_receiver) =
            mpsc::channel(self.response_channel_limit);

        let mut backend =
            ChannelBackend::new(device, response_sender, request_receiver);
        backend.set_hard_eof(self.hard_eof);

        let mut deserializer = Deserializer::new(ChannelSource::new(
            request_sender,
            response_receiver,
        ));

        let block_handle =
            task::spawn_blocking(move || T::deserialize(&mut deserializer));

        backend.run().await?;
        match block_handle.await {
            Ok(actual_result) => actual_result,
            Err(error) => panic::resume_unwind(error.into_panic()),
        }
    }

    pub fn deserialize_buffer<'de, T>(&self, buf: &[u8]) -> Result<T, Error>
    where
        T: Deserialize<'de>,
    {
        let mut deserializer = Deserializer::new(BufferSource::new(buf));
        let value = T::deserialize(&mut deserializer)?;
        if self.hard_eof {
            deserializer.source().ensure_eof()?;
        }
        Ok(value)
    }
}

pub async fn deserialize<'de, T, R>(device: R) -> Result<T, Error>
where
    R: AsyncRead + Unpin,
    T: Deserialize<'de> + Send + 'static,
{
    Config::default().deserialize(device).await
}

pub fn deserialize_buffer<'de, T>(buf: &[u8]) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    Config::default().deserialize_buffer(buf)
}
