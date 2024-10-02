mod internal;
mod public;

#[cfg(test)]
mod test;

pub use public::{deserialize, deserialize_buffer, Config, ConfigError, Error};
