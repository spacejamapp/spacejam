//! log related stuffs

use crate::Arch;

/// Log message from a node
#[derive(Debug, Clone)]
pub struct Message {
    /// The architecture of the node
    pub arch: Arch,
    /// The name of the node
    pub name: String,
    /// The stream type (stdout or stderr)
    pub stream: Stream,
    /// The log line content
    pub content: String,
}

/// stream type
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Stream {
    /// stdout
    Stdout,
    /// stderr
    Stderr,
    /// terminated
    Terminated,
}
