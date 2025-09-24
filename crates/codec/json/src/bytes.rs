use crate::{Json, String, Vec, format};
use anyhow::Result;

impl Json<String> for Vec<u8> {
    fn to_json(self) -> String {
        format!("0x{}", hex::encode(self))
    }

    fn from_json(json: String) -> Result<Self> {
        let bytes = hex::decode(json.trim_start_matches("0x"))
            .map_err(|e| anyhow::anyhow!("failed to decode json string: {e:?}"))?;
        Ok(bytes)
    }
}
