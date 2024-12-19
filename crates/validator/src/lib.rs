//! Extrinsic extensions in SpaceJam

mod extrinsic;
mod result;
mod validator;

pub use {
    extrinsic::{ExtrinsicInMem, ExtrinsicInPool},
    result::{Error, Result},
    validator::Validator,
};

/// Extrinsic type
#[derive(Debug, PartialEq, Eq)]
pub enum ExtrinsicType {
    Assurances,
    Disputes,
    Preimages,
    Guarantees,
    Tickets,
}
