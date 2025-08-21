//! Erasure coding test vectors

use anyhow::Result;

include!(concat!(env!("OUT_DIR"), "/erasure.rs"));

pub fn run(test: &specjam::Test) -> anyhow::Result<()> {
    let mut data = hex::decode(test.input.trim_start_matches("0x"))?;
    let shards = serde_json::from_str::<Vec<String>>(&test.output)?
        .into_iter()
        .map(|s| hex::decode(s.trim_start_matches("0x")).map_err(Into::into))
        .collect::<anyhow::Result<Vec<_>>>()?;

    let rt = tokio::runtime::Runtime::new().unwrap();
    {
        // test encoding
        let edata = data.clone();
        let eshards = shards.clone();
        rt.block_on(async move {
            let encoded = erasure::encode(edata).await.expect("failed to encode");
            assert_eq!(encoded, eshards);
        });
    }

    {
        rt.block_on(async move {
            let decoded = erasure::decode(vec![(0, shards[0].clone()), (2, shards[2].clone())])
                .await
                .expect("failed to decode");
            data.resize(decoded.len(), 0);
            assert_eq!(decoded, data);
        });
    }
    Ok(())
}
