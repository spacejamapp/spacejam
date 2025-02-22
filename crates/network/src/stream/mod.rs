//! Streams for the network.

/// The stream types in JAM.
pub enum Stream {
    /// The stream type for the block announcement.
    BlockAnnouncement,

    /// Unknown stream type.
    Unknown(u8),
}

impl From<u8> for Stream {
    fn from(value: u8) -> Self {
        match value {
            0 => Stream::BlockAnnouncement,
            _ => Stream::Unknown(value),
        }
    }
}
