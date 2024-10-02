mod internal;
mod public;

#[cfg(test)]
mod test;

pub use public::{
    serialize,
    serialize_into_buffer,
    serialize_on_buffer,
    Config,
    ConfigError,
    Error,
};
