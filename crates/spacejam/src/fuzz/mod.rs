//! Fuzz related implementations

use crate::fuzz::message::{Message, Version};
use anyhow::Result;
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
    patch: 1,
};

/// The protocol version of spacejam
pub const PROTOCOL_VERSION: Version = Version {
    major: 0,
    minor: 6,
    patch: 6,
};

/// Extension methods for streams
pub trait StreamExt {
    /// Read a message from the stream
    fn read_message(&mut self) -> Result<Message>;

    /// Write a message to the stream
    fn write_message(&mut self, message: Message) -> Result<()>;
}

impl StreamExt for UnixStream {
    fn read_message(&mut self) -> Result<Message> {
        let mut length = [0; 4];
        self.read_exact(&mut length)?;
        let length = u32::from_le_bytes(length) as usize;
        tracing::debug!("Reading message with length: {length:?}");

        // decode the message from the stream
        let mut bytes = vec![0; length];
        self.read_exact(&mut bytes)?;
        let message = codec::decode(&bytes)?;
        tracing::debug!("Decoded message: {:#?}", message);
        Ok(message)
    }

    fn write_message(&mut self, message: Message) -> Result<()> {
        let bytes = codec::encode(&message)?;
        let length = bytes.len().to_le_bytes().to_vec();
        self.write_all(&[length, bytes].concat())?;
        self.flush()?;
        Ok(())
    }
}
