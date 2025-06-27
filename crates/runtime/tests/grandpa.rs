use score::{
    block::{Head, Header},
    TimeSlot,
};
use spacejam_runtime::{
    storage::{MemoryDb, SyncStorage},
    Grandpa, Handshake,
};
use std::sync::Arc;

#[test]
fn test_select_best_head() {
    let db = MemoryDb::default();
    let ancestry = Arc::new(db);
    let mut grandpa = Grandpa::new(ancestry);
    let mut parent = Header {
        slot: 0,
        parent: [0; 32],
        ..Default::default()
    };

    grandpa.handshake.head = Head {
        slot: 0,
        hash: parent.hash().unwrap(),
    };
    grandpa.ancestry.set_header(&parent).unwrap();
    for i in 1..20u8 {
        let header = Header {
            slot: i as TimeSlot,
            parent: parent.hash().unwrap(),
            ..Default::default()
        };
        let hash = header.hash().unwrap();
        grandpa.add_leaf(header.clone()).unwrap();
        let best = grandpa.select_best_head().unwrap();
        assert_eq!(
            hex::encode(best.best.hash.as_ref()),
            hex::encode(hash.as_ref())
        );

        parent = header;
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
