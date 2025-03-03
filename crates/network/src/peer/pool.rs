//! Peer connection.

use crate::peer::Sync;
use std::{collections::HashMap, ops::Deref, sync::Arc};
use tokio::sync::RwLock;

/// Pool of connections.
pub type Pool = Arc<RwLock<HashMap<[u8; 32], Arc<RwLock<Connection>>>>>;

/// Connection of a peer.
#[derive(Clone)]
pub struct Connection {
    /// The sync information.
    pub sync: Sync,

    /// The connection.
    pub conn: quinn::Connection,
}

impl Connection {
    /// Create a new connection.
    pub fn new(conn: quinn::Connection) -> Self {
        Self {
            sync: Default::default(),
            conn,
        }
    }
}

impl From<quinn::Connection> for Connection {
    fn from(conn: quinn::Connection) -> Self {
        Self::new(conn)
    }
}

impl Deref for Connection {
    type Target = quinn::Connection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}
