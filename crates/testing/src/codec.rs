//! Codec tests

use anyhow::Result;
use core::{AvailAssurance, AvailAssuranceJson};
use scale::Encode;

#[ignore = "implement the jamcodec"]
#[test]
fn decode_avail_assurance() -> Result<()> {
    let json = include_str!("../jamtestvectors/codec/data/assurances_extrinsic.json");
    let data = include_bytes!("../jamtestvectors/codec/data/assurances_extrinsic.bin");

    let assurances: Vec<AvailAssurance> = serde_json::from_str::<Vec<AvailAssuranceJson>>(json)?
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<_>>>()?;

    assert_eq!(assurances.encode(), data);
    Ok(())
}
