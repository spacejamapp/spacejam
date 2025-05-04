//! Services for the JAM index

use crate::Config;

mod node;

/// Start the Jadex service
pub async fn start<Hook: runtime::Hook + Default + Send + Sync + 'static>(
    config: &Config,
    hook: Hook,
) -> anyhow::Result<()> {
    node::start(config, hook).await
}
