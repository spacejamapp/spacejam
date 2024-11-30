//! Codec tests

use anyhow::Result;
use core::AvailAssurance;
use serde_json::Value;

#[test]
fn decode_avail_assurance() -> Result<()> {
    let json = include_str!("../jamtestvectors/codec/data/assurances_extrinsic.json");
    let _data = include_bytes!("../jamtestvectors/codec/data/assurances_extrinsic.bin");

    let arr: Vec<Value> = serde_json::from_str(json)?;
    let _assurances: Vec<AvailAssurance> = arr.iter().map(|_| todo!()).collect();

    Ok(())
}
