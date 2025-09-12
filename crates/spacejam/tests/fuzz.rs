//! Fuzz tests

use spacejam::fuzz::message::{Message, PeerInfo};

#[test]
fn fuzz_message_encoding() {
    let message = Message::Info(PeerInfo::default());
    let encoded = codec::encode(&message).unwrap();
    let decoded = codec::decode(&encoded).unwrap();
    assert_eq!(message, decoded);
}
