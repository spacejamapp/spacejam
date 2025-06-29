//! Fuzz related implementations

use crate::fuzz::message::Version;

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
