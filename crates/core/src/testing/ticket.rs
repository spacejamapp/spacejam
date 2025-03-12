//! Ticket testing

use super::Node;
use crate::OpaqueHash;

#[tokio::test]
async fn series() {
    let node = Node::genesis(OpaqueHash::default())
        .await
        .expect("failed to create genesis node");

    let (mut next, _) = node.runtime.next().await.expect("failed to get next");
    node.runtime
        .finalize(&mut next)
        .await
        .expect("failed to finalize");
}
