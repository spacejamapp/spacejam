//! Json implementation for array

use crate::{Json, String, Vec, format};
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

impl<const L: usize> Json<String> for [u8; L] {
    fn to_json(self) -> String {
        format!("0x{}", hex::encode(self))
    }

    fn from_json(json: String) -> Result<Self> {
        let bytes = hex::decode(json.trim_start_matches("0x"))
            .map_err(|e| anyhow::anyhow!("failed to decode json string: {e:?}"))?;
        let actual = bytes.len();
        bytes.try_into().map_err(|_: Vec<u8>| {
            anyhow::anyhow!("Invalid hex string, target length is {L}, actual length is {actual}")
        })
    }
}

impl<M: Serialize + DeserializeOwned, T: Json<M>, const L: usize> Json<Vec<M>> for [T; L] {
    fn to_json(self) -> Vec<M> {
        self.into_iter().map(|v| v.to_json()).collect()
    }

    fn from_json(json: Vec<M>) -> Result<Self> {
        let mut array = Vec::with_capacity(L);
        for v in json {
            array.push(T::from_json(v)?);
        }
        array
            .try_into()
            .map_err(|_: Vec<T>| anyhow::anyhow!("Invalid array length"))
    }
}

#[cfg(feature = "codec")]
impl<M: Serialize + DeserializeOwned, T: Json<M>, const N: usize> Json<Vec<M>>
    for codec::FixedArray<T, N>
{
    fn to_json(self) -> Vec<M> {
        self.into_iter().map(|v| v.to_json()).collect()
    }

    fn from_json(json: Vec<M>) -> Result<Self> {
        let v: Vec<T> = json.into_iter().map(T::from_json).collect::<Result<_>>()?;
        codec::FixedArray::try_from_vec(v)
            .map_err(|_| anyhow::anyhow!("Invalid array length, expected {N}"))
    }
}

#[cfg(feature = "codec")]
impl<M: Serialize + DeserializeOwned, T: Json<M>, const N: usize> Json<Vec<M>>
    for codec::Array<T, N>
{
    fn to_json(self) -> Vec<M> {
        self.into_iter().map(|v| v.to_json()).collect()
    }

    fn from_json(json: Vec<M>) -> Result<Self> {
        let v: Vec<T> = json.into_iter().map(T::from_json).collect::<Result<_>>()?;
        codec::Array::try_from_vec(v)
            .map_err(|_| anyhow::anyhow!("Invalid array length, expected {N}"))
    }
}
