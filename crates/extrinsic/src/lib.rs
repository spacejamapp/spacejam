//! Extrinsic extensions in SpaceJam

mod extrinsic;
mod pool;
mod result;
mod validator;

pub use {
    pool::Pool,
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
