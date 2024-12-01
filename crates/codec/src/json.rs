//! JSON utilities
//!
//! Now using hex as the default encoding.
use anyhow::Result;

/// A trait for types that can be encoded and decoded to and from JSON.
pub trait Json<Target>: Sized + std::fmt::Debug {
    /// Converts the value to its JSON representation.
    fn to_json(self) -> Target;

    /// Converts the value from its JSON representation.
    fn from_json(json: Target) -> Result<Self>;
}

impl<M, N> Json<Option<M>> for Option<N>
where
    N: Json<M>,
{
    fn to_json(self) -> Option<M> {
        self.map(|v| v.to_json())
    }

    fn from_json(json: Option<M>) -> Result<Self> {
        json.map(|v| N::from_json(v)).transpose()
    }
}

impl<M, N> Json<Vec<M>> for Vec<N>
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

impl Json<String> for Vec<u8> {
    fn to_json(self) -> String {
        format!("0x{}", hex::encode(self))
    }

    fn from_json(json: String) -> Result<Self> {
        let bytes = hex::decode(json.trim_start_matches("0x"))?;
        Ok(bytes)
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

impl_bytes!(1, 2, 3, 4, 6, 8, 16, 32, 64, 96, 128, 144, 256, 784);

macro_rules! impl_array {
    ($($len:expr),*) => {
        $(
            impl<M, N: Default + Copy> Json<Vec<M>> for [N; $len]
            where
                N: Json<M>,
            {
                fn to_json(self) -> Vec<M> {
                    self.into_iter().map(|v| v.to_json()).collect()
                }

                fn from_json(json: Vec<M>) -> Result<Self> {
                    let mut array = [N::default(); $len];
                    for (i, v) in json.into_iter().enumerate() {
                        array[i] = N::from_json(v)?;
                    }
                    Ok(array)
                }
            }
        )*
    };
}

impl_array!(1, 2, 3, 4, 6, 8, 16, 32, 64, 96, 128, 144, 256, 784);

macro_rules! impl_primitive {
    ($($ty:ty),*) => {
        $(
            impl Json<$ty> for $ty {
                fn to_json(self) -> $ty {
                    self
                }

                fn from_json(json: $ty) -> Result<Self> {
                    Ok(json)
                }
            }
        )*
    };
}

impl_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, bool, ());
