//! Fuzz tests

use spacejam::fuzz::message::{Message, PeerInfo, Version};

#[test]
fn fuzz_message_encoding() {
    let message = Message::Info(PeerInfo::default());
    let encoded = codec::encode(&message).unwrap();
    let decoded = codec::decode(&encoded).unwrap();
    assert_eq!(message, decoded);
}

#[test]
fn peer_info() {
    let info = PeerInfo {
        fuzz_version: 1,
        fuzz_features: 2,
        jam_version: Version {
            major: 0,
            minor: 7,
            patch: 0,
        },
        app_version: Version {
            major: 0,
            minor: 1,
            patch: 25,
        },
        app_name: "fuzzer".to_string(),
    };

    let encoded = codec::encode(&Message::Info(info)).unwrap();
    let expected = include_bytes!(
        "../../../res/jam-conformance/fuzz-proto/examples/v1/00000000_fuzzer_peer_info.bin"
    );
    assert_eq!(encoded, expected);
}
