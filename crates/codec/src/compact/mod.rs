//! Number encoding and decoding

pub mod num;
pub mod vlen;

pub use {
    crate::with::compact::{deserialize, serialize},
    num::Numeric,
    vlen::{decode, decode_from, encode},
};
