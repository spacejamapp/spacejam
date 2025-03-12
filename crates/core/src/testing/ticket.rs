//! Ticket testing

use super::Node;
use crate::{extrinsic::TicketsOrKeys, runtime::Storage, OpaqueHash};

#[tokio::test]
async fn empty_series() {
    let node = Node::genesis(OpaqueHash::default())
        .await
        .expect("failed to create genesis node");

    let mut count = 0;
    loop {
        let (mut next, _) = node.author(count + 1).await.expect("failed to author");
        if let Err(e) = node.runtime.finalize(&mut next).await {
            panic!("failed to finalize: {e}, count: {count}");
        }

        let safrole = node
            .runtime
            .storage
            .safrole()
            .expect("failed to get safrole");

        // safrole ticket accumulator never exceed the maxium attempts
        // of a single validator.
        assert!(safrole.accumulator.len() <= 2);

        // no tickets mark since we just have one validator
        assert!(next.header.tickets_mark.is_none());

        // the blocks are always sealed with fallback keys
        assert!(matches!(safrole.series, TicketsOrKeys::Keys(_)));

        count += 1;
        if count >= crate::EPOCH_LENGTH {
            break;
        }
    }
}
