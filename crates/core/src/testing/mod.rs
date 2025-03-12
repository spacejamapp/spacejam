//! Testing utilities
#![cfg(test)]

use crate::{
    extrinsic::TicketBody,
    runtime::{storage::BlockStorage, Head, Validator},
    OpaqueHash,
};
use node::Node;

mod node;
mod ticket;
mod validator;

#[tokio::test]
async fn genesis() {
    let node = Node::genesis(OpaqueHash::default())
        .await
        .expect("failed to create genesis node");
    let block = node.genesis;

    // 1. check the block is stored
    let hash = block.hash().unwrap();
    let sblock = node.runtime.storage.get_block(&hash).unwrap();
    assert_eq!(block, sblock);

    // 2.check the latest finalized head is recorded
    let finalized = node.runtime.storage.get_finalized().unwrap();
    assert_eq!(finalized.hash, hash);
    assert_eq!(finalized.slot, block.header.slot);

    // 3. check the grandpa is updated
    let grandpa = node.runtime.grandpa.read().await;
    assert!(grandpa.handshake.leaves.is_empty());
    assert_eq!(grandpa.handshake.head, finalized);
    assert_eq!(
        grandpa.grid.next.to_vec(),
        node.validators
            .iter()
            .map(|v| v.ed25519_public_key())
            .collect::<Vec<_>>()
    );

    // 4. check the ancestry is set up
    let ancestry = grandpa.ancestors(&hash, block.header.parent);
    assert!(ancestry.is_empty());
    assert!(grandpa.is_descendant_of(hash, block.header.parent));
}

#[tokio::test]
async fn author() {
    // create the genesis block
    let node = Node::genesis(OpaqueHash::default())
        .await
        .expect("failed to create genesis node");
    let block = node.genesis;
    // 1. get the next block
    let (next, ticket) = node.runtime.next().await.expect("failed to get next block");
    assert_eq!(next.header.parent, block.header.hash().unwrap());

    // 2. verify the ticket
    //
    // NOTE: we don't always have a ticket since block authoring is slot based.
    if let Some(ticket) = ticket {
        assert_eq!(ticket.attempt, 0);

        // 2.1. verify the ticket signature
        let message = TicketBody::message(ticket.attempt, &[0; 32]);
        let keys = &node
            .validators
            .iter()
            .map(|v| v.bandersnatch_public_key())
            .collect::<Vec<_>>();
        let verifier = crypto::ring::verifier(keys.clone());
        verifier
            .ring_vrf_verify(&message, &[], &ticket.signature)
            .expect("failed to verify the ticket");
    }

    // 3. the expool contains no ticket
    assert!(next.header.tickets_mark.is_none());
}

#[tokio::test]
async fn finalize() {
    let node = Node::genesis(OpaqueHash::default())
        .await
        .expect("failed to create genesis node");

    // 1. author a block
    let (mut next, _) = node.runtime.next().await.expect("failed to get next block");

    // 2. finalize the block
    node.runtime
        .finalize(&mut next)
        .await
        .expect("failed to finalize block");

    // 3. check grandpa is updated
    let grandpa = node.runtime.grandpa.read().await;
    let head = Head {
        hash: next.hash().unwrap(),
        slot: next.header.slot,
    };
    assert_eq!(grandpa.handshake.head, head);
    assert!(grandpa.is_descendant_of(head.hash, node.genesis.header.parent));
}
