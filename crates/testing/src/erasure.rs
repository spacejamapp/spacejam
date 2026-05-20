//! Erasure coding test vectors

use anyhow::Result;

include!(concat!(env!("OUT_DIR"), "/erasure.rs"));

pub async fn run(test: &specjam::Test) -> anyhow::Result<()> {
    let mut data = hex::decode(test.input.trim_start_matches("0x"))?;
    let shards = serde_json::from_str::<Vec<String>>(&test.output)?
        .into_iter()
        .map(|s| hex::decode(s.trim_start_matches("0x")).map_err(Into::into))
        .collect::<anyhow::Result<Vec<_>>>()?;

    // test encoding
    let edata = data.clone();
    let encoded = erasure::encode(edata).await.expect("failed to encode");
    assert_eq!(encoded, shards);

    // test decoding — provide the minimum original shards needed
    let decode_shards: Vec<_> = shards
        .into_iter()
        .enumerate()
        .take(erasure::Config::default().original)
        .collect();
    let decoded = erasure::decode(decode_shards)
        .await
        .expect("failed to decode");
    data.resize(decoded.len(), 0);
    assert_eq!(decoded, data);
    Ok(())
}
