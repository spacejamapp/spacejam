//! Authorization related stuffs

use score::{ServiceId, service::Authorizer};

/// Authorization related stuffs
pub struct Auth {
    /// The authorization token
    pub token: Vec<u8>,

    /// The authorization host
    pub host: ServiceId,

    /// The authorizer
    pub authorizer: Authorizer,
}
