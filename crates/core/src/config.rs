//! The configuration of SpaceJam

use crate::{state::Storage, validator::Validator};

/// The configuration of SpaceJam
pub trait Config {
    /// The validator of SpaceJam
    type Validator: Validator + Default;

    /// The database of SpaceJam
    type Db: Storage;
}
