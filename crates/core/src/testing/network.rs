//! Testing network in memory

use crate::{
    extrinsic::TicketsOrKeys,
    runtime::Validator,
    safrole::ValidatorsData,
    testing::{self, Node, TEST_VALIDATORS},
    Ed25519Public,
};
use anyhow::Result;
use std::collections::HashMap;

/// The testing network
pub struct Network {
    /// The nodes
    pub nodes: HashMap<Ed25519Public, Node>,

    /// The validators
    pub validators: ValidatorsData,
}

impl Network {
    /// Initialize a new network
    pub async fn init() -> Result<Self> {
        testing::setup_tracing();
        let mut nodes = HashMap::new();
        let validators = testing::validators();
        for seed in TEST_VALIDATORS {
            let node = Node::genesis(seed, validators.clone()).await?;
            nodes.insert(node.runtime.validator.ed25519_public_key(), node);
        }
        Ok(Self { nodes, validators })
    }

    /// Run to the next slot
    pub async fn next(&mut self, timeslot: u32) -> Result<()> {
        for node in self.nodes.values_mut() {
            let mut author = node.author();
            let (_block, _ticket) = author.on_timeslot(timeslot).await?;

            // TODO: subscribe blocks and tickets to network
        }
        Ok(())
    }
}

/// The public keys of the testing validators
pub const ED25519_PUBLIC_KEYS: &[Ed25519Public] = &[
    [
        129, 57, 119, 14, 168, 125, 23, 95, 86, 163, 84, 102, 195, 76, 126, 204, 203, 141, 138,
        145, 180, 238, 55, 162, 93, 246, 15, 91, 143, 201, 179, 148,
    ],
    [
        202, 147, 172, 23, 5, 24, 112, 113, 214, 123, 131, 199, 255, 14, 254, 129, 8, 232, 236, 69,
        48, 87, 93, 119, 38, 135, 147, 51, 219, 218, 190, 124,
    ],
    [
        138, 136, 227, 221, 116, 9, 241, 149, 253, 82, 219, 45, 60, 186, 93, 114, 202, 103, 9, 191,
        29, 148, 18, 27, 243, 116, 136, 1, 180, 15, 111, 92,
    ],
    [
        59, 106, 39, 188, 206, 182, 164, 45, 98, 163, 168, 208, 42, 111, 13, 115, 101, 50, 21, 119,
        29, 226, 67, 166, 58, 192, 72, 161, 139, 89, 218, 41,
    ],
    [
        237, 73, 40, 198, 40, 209, 194, 198, 234, 233, 3, 56, 144, 89, 149, 97, 41, 89, 39, 58, 92,
        99, 249, 54, 54, 193, 70, 20, 172, 135, 55, 209,
    ],
    [
        110, 122, 28, 221, 41, 176, 183, 143, 209, 58, 244, 197, 89, 143, 239, 244, 239, 42, 151,
        22, 110, 60, 166, 242, 228, 251, 252, 205, 128, 80, 91, 241,
    ],
];

#[tokio::test]
async fn verify_tickets() {
    let network = Network::init().await.unwrap();
    let node0 = network.nodes.get(&ED25519_PUBLIC_KEYS[0]).unwrap();
    let node1 = network.nodes.get(&ED25519_PUBLIC_KEYS[1]).unwrap();
    assert_eq!(node0.validators.len(), 6);
    assert_eq!(node1.validators.len(), 6);

    let mut author0 = node0.author();
    let mut author1 = node1.author();
    author0.on_new_epoch().await.unwrap();
    author1.on_new_epoch().await.unwrap();

    assert_eq!(author0.validators.len(), 6);
    assert_eq!(author1.validators.len(), 6);
    let ticket0 = author0.ticket().await.unwrap().unwrap();
    let ticket1 = author1.ticket().await.unwrap().unwrap();

    author0.insert_ticket(ticket1.1).await.unwrap();
    author1.insert_ticket(ticket0.1).await.unwrap();
}

#[tokio::test]
async fn verify_headers() {
    let network = Network::init().await.unwrap();
    let node0 = network.nodes.get(&ED25519_PUBLIC_KEYS[0]).unwrap();
    let node1 = network.nodes.get(&ED25519_PUBLIC_KEYS[1]).unwrap();

    let mut author0 = node0.author();
    let mut author1 = node1.author();
    author0.on_new_epoch().await.unwrap();
    author1.on_new_epoch().await.unwrap();

    // assert that the series is shared
    assert_eq!(&author0.series, &author1.series);
    let TicketsOrKeys::Keys(keys) = &author0.series else {
        panic!("series should be keys");
    };

    // assert that the keys are in the series
    let mut n0 = vec![];
    let mut n1 = vec![];
    for (i, key) in keys.iter().enumerate() {
        if key == &author0.me {
            n0.push(i as u32);
        } else if key == &author1.me {
            n1.push(i as u32);
        }
    }

    assert_eq!(n0, vec![2]);
    assert_eq!(n1, vec![4]);

    // author 0 should author at slot 2
    author0.timeslot = 2;
    let header0 = author0.author().await.unwrap();
    assert_eq!(header0.slot, 2);

    // author 1 should author at slot 4
    author1.timeslot = 4;
    let header1 = author1.author().await.unwrap();
    assert_eq!(header1.slot, 4);

    // verify headers
    let importer0 = node0.importer();
    let importer1 = node1.importer();
    importer0.validate(&header1).await.unwrap();
    importer1.validate(&header0).await.unwrap();
}
