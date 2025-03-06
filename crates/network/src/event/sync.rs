//! Handler of sync events

use crate::Network;
use score::{block::Header, runtime::Head};

/// Announce a block to the network
pub async fn announce<C: score::runtime::Config>(
    runtime: Network<C>,
    header: Box<Header>,
    head: Head,
) -> anyhow::Result<()> {
    if let Err(e) = runtime.grandpa.read().await.verify(&header).await {
        anyhow::bail!(e);
    }

    // broadcast the block to the network
    runtime.announce.send((*header, head))?;
    Ok(())
}
