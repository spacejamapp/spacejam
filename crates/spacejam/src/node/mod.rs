//! Node for SpaceJam

use spec::NodeSpec;
pub use {builder::Builder, spec::RuntimeSpec};

mod builder;
pub mod spec;

/// SpaceJam node
pub enum SpaceJam<C: spec::RuntimeSpec> {
    /// Authoring blocks per 6 secs without network
    Dev(spec::Dev<C>),

    /// Importing and finalizing blocks with grandpa with JSON-RPC provided
    Light(spec::Light<C>),

    /// Validating blocks and sending tickets
    Validating(spec::Validating<C>),
}

impl<C: spec::RuntimeSpec> SpaceJam<C> {
    pub async fn start(self) -> anyhow::Result<()> {
        match self {
            Self::Dev(spec) => spec.start().await,
            Self::Light(spec) => spec.start().await,
            Self::Validating(spec) => spec.start().await,
        }
    }
}
