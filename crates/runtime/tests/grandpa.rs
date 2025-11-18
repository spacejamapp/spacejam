use score::block::Head;
use spacejam_runtime::Handshake;

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

    let encoded = codec::encode(&handshake);
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
