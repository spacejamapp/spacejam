//! Preimage request stream.

use crate::Network;
use quinn::{RecvStream, SendStream};
use score::OpaqueHash;

/// Send a preimage request.
#[allow(unused)]
pub async fn send(
    mut send: SendStream,
    recv: RecvStream,
    request: OpaqueHash,
) -> anyhow::Result<()> {
    let mut buf = vec![143];
    buf.extend_from_slice(&codec::encode(&request));
    send.write_all(&buf).await?;
    send.finish()?;
    Ok(())
}

impl<C: runtime::Config> Network<C> {
    /// Receive a preimage request.
    pub async fn recv_ce143(
        &self,
        mut send: SendStream,
        mut recv: RecvStream,
    ) -> anyhow::Result<()> {
        let mut hash = [0; 32];
        recv.read_exact(&mut hash).await?;

        // fetch the preimage
        // let preimage = runtime.runtime.storage.fetch_preimage(hash)?;
        //
        // TODO: fetch the preimage from the storage
        let preimage = vec![];
        send.write_all(&preimage).await?;
        send.finish()?;
        Ok(())
    }
}
