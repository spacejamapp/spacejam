use codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// Represents an activity record.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Encode, Decode, Serialize, Deserialize)]
pub struct ValidatorActivityRecord {
    /// (b) Number of blocks produced
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub blocks: u32,

    /// (t) Number of tickets
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub tickets: u32,

    /// (p) Number of pre-images
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub pre_images: u32,

    /// (d) Size of pre-images
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub pre_images_size: u32,

    /// (g) Number of guarantees
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub guarantees: u32,

    /// (a) Number of assurances
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub assurances: u32,
}

/// Represents a core record.
#[derive(Debug, PartialEq, Eq, Clone, Default, Encode, Decode, Serialize, Deserialize)]
pub struct CoreActivityRecord {
    /// Total gas consumed by core for reported work. Includes all refinement and authorizations.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub gas_used: u64,

    /// Number of segments imported from DA made by core for reported work.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub imports: u16,

    /// Total number of extrinsics used by core for reported work.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub extrinsic_count: u16,

    /// Total size of extrinsics used by core for reported work.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub extrinsic_size: u32,

    /// Number of segments exported into DA made by core for reported work.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub exports: u16,

    /// The work-bundle size. This is the size of data being placed into Audits DA by the core.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub bundle_size: u32,

    /// Amount of bytes which are placed into either Audits or Segments DA.
    /// This includes the work-bundle (including all extrinsics and imports) as well as all
    /// (exported) segments.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub da_load: u64,

    /// Number of validators which formed super-majority for assurance.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub popularity: u64,
}

/// Represents a service record.
#[derive(Debug, PartialEq, Eq, Clone, Default, Encode, Decode, Serialize, Deserialize)]
pub struct ServiceActivityRecord {
    /// Number of preimages provided to this service.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub provided_count: u16,
    /// Total size of preimages provided to this service.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub provided_size: u32,
    /// Number of work-items refined by service for reported work.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub refinement_count: u32,
    /// Amount of gas used for refinement by service for reported work.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub refinement_gas_used: u64,
    /// Number of segments imported from the DL by service for reported work.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub imports: u32,
    /// Number of segments exported into the DL by service for reported work.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub exports: u32,
    /// Total size of extrinsics used by service for reported work.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub extrinsic_size: u32,
    /// Total number of extrinsics used by service for reported work.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub extrinsic_count: u32,
    /// Number of work-items accumulated by service.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub accumulate_count: u32,
    /// Amount of gas used for accumulation by service.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub accumulate_gas_used: u64,
    /// Number of transfers processed by service.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
    pub on_transfers_count: u32,
    /// Amount of gas used for processing transfers by service.
    #[codec(compact)]
    #[serde(with = "serde_jam::compact")]
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
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ];

    let stat = Statistics::decode(&mut &encoded[..]).unwrap();

    // test val
    {
        let encoded = stat.vals_current[0].encode();
        let decoded: ValidatorActivityRecord = serde_jam::decode(&encoded).unwrap();
        assert_eq!(decoded, stat.vals_current[0]);

        // encode tests
        let sencoded = serde_jam::encode(&decoded).unwrap();
        assert_eq!(sencoded, encoded);
    }

    // test core
    {
        let encoded = stat.cores[0].encode();
        let decoded: CoreActivityRecord = serde_jam::decode(&encoded).unwrap();
        assert_eq!(decoded, stat.cores[0]);

        // encode tests
        let sencoded = serde_jam::encode(&decoded).unwrap();
        assert_eq!(sencoded, encoded);
    }

    // test stat
    {
        let encoded = stat.encode();
        let decoded: Statistics = serde_jam::decode(&encoded).unwrap();
        assert_eq!(decoded, stat);

        // encode tests
        let sencoded = serde_jam::encode(&decoded).unwrap();
        assert_eq!(sencoded, encoded);
    }

    println!("=== testing statistics ===");

    // test jam codec matches
    {
        let jencoded = stat.encode();
        println!("jencoded: {}", jencoded.len());
        assert_eq!(jencoded, encoded);
    }

    // test stat dir
    {
        let decoded: Statistics = serde_jam::decode(&encoded).unwrap();
        assert_eq!(decoded, stat);
        println!("{:#?}", decoded);

        // encode tests
        let sencoded = serde_jam::encode(&decoded).unwrap();
        println!("sencoded: {:?}", sencoded.len());
        assert_eq!(sencoded, encoded);
    }

    /*  // test service
    {
        let encoded = stat.services[0].1.encode();
        let decoded: ServiceActivityRecord = serde_jam::decode(&encoded).unwrap();
        assert_eq!(decoded, stat.services[0].1);
    } */
}
