//! The runtime of SpaceJam
use score::block::{Block, BlocksHistory};
use validation::Validation;
use validator::validate::ValidateExtrinsic;

pub mod validation;

/// The runtime of SpaceJam
pub struct SpaceJam<Validator: ValidateExtrinsic> {
    /// The blocks history of the SpaceJam
    pub history: BlocksHistory,

    /// The validation service
    pub validation: Validation<Validator>,
}

impl<Validator: ValidateExtrinsic> SpaceJam<Validator> {
    /// Import a new block into the chain
    pub async fn import(&mut self, _block: Block) {}
}
