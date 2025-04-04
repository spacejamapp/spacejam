//! Handshake data

use crate::grandpa::Head;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Head and unfinalized leaves of the grandpa protocol.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Handshake {
    /// The hash of the head of the chain, e.g. the finalized header.
    ///
    /// This represents the latest block that has been finalized by the GRANDPA protocol.
    pub head: Head,

    /// The leaves of the best finalized head.
    ///
    /// Descendants of the latest finalized block with no known children.
    pub leaves: HashSet<Head>,
}

impl Handshake {
    /// Create a new handshake from the given head.
    pub fn new(head: Head) -> Self {
        Self {
            head,
            leaves: Default::default(),
        }
    }
}

#[test]
fn encoding() {
    let handshake = Handshake {
        head: Head {
            hash: [0; 32],
            slot: 0,
        },
        leaves: vec![
            Head {
                hash: [1; 32],
                slot: 1,
            },
            Head {
                hash: [2; 32],
                slot: 2,
            },
        ]
        .into_iter()
        .collect(),
    };

    let encoded = codec::encode(&handshake).expect("failed to encode handshake");
    let decoded = codec::decode::<Handshake>(&encoded).expect("failed to decode handshake");
    assert_eq!(handshake, decoded);

    // test handwrite encoding
    let mut buf = vec![];
    buf.extend_from_slice(&handshake.head.hash);
    buf.extend_from_slice(&handshake.head.slot.to_le_bytes());
    buf.push(handshake.leaves.len() as u8);
    for leaf in handshake.leaves.iter() {
        buf.extend_from_slice(&leaf.hash);
        buf.extend_from_slice(&leaf.slot.to_le_bytes());
    }

    assert_eq!(encoded, buf);
}
