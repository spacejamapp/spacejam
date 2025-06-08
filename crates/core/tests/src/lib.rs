use codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// Represents an activity record.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Encode, Decode, Serialize, Deserialize)]
pub struct ValidatorActivityRecord {
    /// (b) Number of blocks produced
    pub blocks: u32,

    /// (t) Number of tickets
    pub tickets: u32,

    /// (p) Number of pre-images
    pub pre_images: u32,

    /// (d) Size of pre-images
    pub pre_images_size: u32,

    /// (g) Number of guarantees
    pub guarantees: u32,

    /// (a) Number of assurances
    pub assurances: u32,
}

/// Represents a core record.
#[derive(Debug, PartialEq, Eq, Clone, Default, Encode, Decode, Serialize, Deserialize)]
pub struct CoreActivityRecord {
    /// Amount of bytes which are placed into either Audits or Segments DA.
    /// This includes the work-bundle (including all extrinsics and imports) as well as all
    /// (exported) segments.
    #[codec(compact)]
    pub da_load: u32,

    /// Number of validators which formed super-majority for assurance.
    #[codec(compact)]
    pub popularity: u16,

    /// Number of segments imported from DA made by core for reported work.
    #[codec(compact)]
    pub imports: u16,

    /// Number of segments exported into DA made by core for reported work.
    #[codec(compact)]
    pub exports: u16,

    /// Total size of extrinsics used by core for reported work.
    #[codec(compact)]
    pub extrinsic_size: u32,

    /// Total number of extrinsics used by core for reported work.
    #[codec(compact)]
    pub extrinsic_count: u16,

    /// The work-bundle size. This is the size of data being placed into Audits DA by the core.
    #[codec(compact)]
    pub bundle_size: u32,

    /// Total gas consumed by core for reported work. Includes all refinement and authorizations.
    #[codec(compact)]
    pub gas_used: u64,
}

/// Represents a service record.
#[derive(Debug, PartialEq, Eq, Clone, Default, Encode, Decode, Serialize, Deserialize)]
pub struct ServiceActivityRecord {
    /// Number of preimages provided to this service.
    #[codec(compact)]
    pub provided_count: u16,
    /// Total size of preimages provided to this service.
    #[codec(compact)]
    pub provided_size: u32,
    /// Number of work-items refined by service for reported work.
    #[codec(compact)]
    pub refinement_count: u32,
    /// Amount of gas used for refinement by service for reported work.
    #[codec(compact)]
    pub refinement_gas_used: u64,
    /// Number of segments imported from the DL by service for reported work.
    #[codec(compact)]
    pub imports: u32,
    /// Number of segments exported into the DL by service for reported work.
    #[codec(compact)]
    pub exports: u32,
    /// Total size of extrinsics used by service for reported work.
    #[codec(compact)]
    pub extrinsic_size: u32,
    /// Total number of extrinsics used by service for reported work.
    #[codec(compact)]
    pub extrinsic_count: u32,
    /// Number of work-items accumulated by service.
    #[codec(compact)]
    pub accumulate_count: u32,
    /// Amount of gas used for accumulation by service.
    #[codec(compact)]
    pub accumulate_gas_used: u64,
    /// Number of transfers processed by service.
    #[codec(compact)]
    pub on_transfers_count: u32,
    /// Amount of gas used for processing transfers by service.
    #[codec(compact)]
    pub on_transfers_gas_used: u64,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Encode, Decode, Serialize, Deserialize)]
pub struct Statistics {
    pub vals_current: [ValidatorActivityRecord; 6],
    pub vals_last: [ValidatorActivityRecord; 6],
    pub cores: [CoreActivityRecord; 2],
    pub services: Vec<(u32, ServiceActivityRecord)>,
}

#[test]
fn codec_stat() {
    let encoded = [
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 6, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 4, 0, 0, 0, 6, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0,
        0, 0, 6, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 6, 0, 0, 0,
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 6, 0, 0, 0, 3, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 129, 108, 192, 42,
        200, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 192, 42, 200, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let stat = Statistics::decode(&mut &encoded[..]).unwrap();
    let pencoded = stat.encode();
    assert_eq!(encoded.to_vec(), pencoded);

    let service = stat.services[0].1.clone();
    println!("{:?}", service);

    let ssvc: score::statistic::ServiceActivityRecord =
        serde_jam::decode(&service.encode()).unwrap();
    println!("{:?}", ssvc);
}
