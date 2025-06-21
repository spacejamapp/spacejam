//! GraphQL configs

use std::net::SocketAddr;

/// Config for graphql
pub struct Graphql {
    /// The graphql server address
    pub graphql: SocketAddr,
}
