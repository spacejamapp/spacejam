//! Fuzz tests

use spacejam::fuzz::message::{Message, PeerInfo, Version};

#[test]
fn fuzz_message_encoding() {
    let message = Message::Info(PeerInfo {
        name: "spacejam".to_string(),
        version: Version {
            major: 0,
            minor: 0,
            patch: 1,
        },
        protocol: Version {
            major: 0,
            minor: 6,
            patch: 7,
        },
    });

    let encoded = codec::encode(&message).unwrap();
    let decoded = codec::decode(&encoded).unwrap();
    assert_eq!(message, decoded);
}
