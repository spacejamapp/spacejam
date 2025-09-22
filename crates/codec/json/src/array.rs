use crate::Json;
use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};

impl<M: Serialize + DeserializeOwned, N> Json<Vec<M>> for Vec<N>
where
    N: Json<M>,
{
    fn to_json(self) -> Vec<M> {
        self.into_iter().map(|v| v.to_json()).collect()
    }

    fn from_json(json: Vec<M>) -> Result<Self> {
        json.into_iter().map(|v| N::from_json(v)).collect()
    }
}

macro_rules! impl_bytes {
    ($($len:expr),*) => {
        $(
            impl Json<String> for [u8; $len] {
                fn to_json(self) -> String {
                    format!("0x{}", hex::encode(self))
                }

                fn from_json(json: String) -> Result<Self> {
                    let bytes = hex::decode(json.trim_start_matches("0x"))?;
                    let len = bytes.len();

                    bytes.try_into().map_err(|_| {
                        anyhow::anyhow!(
                            "Invalid hex string, target length is {len}, actual length is {actual}",
                            len = $len,
                            actual = len
                        )
                    })
                }
            }
        )*
    };
}

impl_bytes!(
    1, 2, 3, 4, 5, 6, 8, 12, 16, 17, 32, 64, 96, 128, 144, 256, 784
);

macro_rules! impl_array {
    ($($len:expr),*) => {
        $(
            impl<M: Serialize + DeserializeOwned, N: Default> Json<Vec<M>> for [N; $len]
            where
                N: Json<M>,
            {
                fn to_json(self) -> Vec<M> {
                    self.into_iter().map(|v| v.to_json()).collect()
                }

                fn from_json(json: Vec<M>) -> Result<Self> {
                    let mut array = Vec::with_capacity($len);
                    for v in json {
                        array.push(N::from_json(v)?);
                    }
                    array.try_into().map_err(|_| anyhow::anyhow!("Invalid array length"))
                }
            }
        )*
    };
}

impl_array!(1, 2, 3, 4, 5, 6, 8, 12, 16, 32, 64, 96, 128, 144, 256, 784);
