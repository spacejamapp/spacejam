//! Fuzz related implementations

use crate::fuzz::message::{Message, Version};
use anyhow::{Context, Result};
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

pub mod fuzzer;
pub mod message;
pub mod target;
pub mod trace;

/// The binary version of spacejam
pub const VERSION: Version = Version {
    major: 0,
    minor: 0,
    patch: 8,
};

/// The protocol version of spacejam
pub const PROTOCOL_VERSION: Version = Version {
    major: 0,
    minor: 7,
    patch: 0,
};

/// Extension methods for streams
pub trait StreamExt {
    /// Read a message from the stream
    fn read_message(&mut self) -> Result<Message>;

    /// Write a message to the stream
    fn write_message(&mut self, message: Message) -> Result<()>;
}

impl StreamExt for UnixStream {
    #[tracing::instrument(skip_all, name = " read", parent = None)]
    fn read_message(&mut self) -> Result<Message> {
        let mut length = [0; 4];
        self.read_exact(&mut length)?;
        let length = u32::from_le_bytes(length) as usize;

        // decode the message from the stream
        let mut bytes = vec![0; length];
        self.read_exact(&mut bytes)?;
        let message =
            codec::decode(&bytes).context(format!("failed to decode message: length={length}"))?;
        tracing::debug!("message(length): {message}");
        Ok(message)
    }

    #[tracing::instrument(skip_all, name = "write", parent = None)]
    fn write_message(&mut self, message: Message) -> Result<()> {
        let bytes = codec::encode(&message)?;
        let length = bytes.len() as u32;

        tracing::debug!("message({length}): {message}");
        self.write_all(&[length.to_le_bytes().to_vec(), bytes].concat())?;
        self.flush()?;
        Ok(())
    }
}
