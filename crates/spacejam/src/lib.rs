//! The runtime of SpaceJam
use score::{
    block::{Block, BlocksHistory},
    validator::validate::ValidateExtrinsic,
};
use validator::Validation;

pub mod cmd;
pub mod validator;

/// The runtime of SpaceJam
pub struct SpaceJam<Validator: ValidateExtrinsic> {
    /// The blocks history of the SpaceJam
    pub history: BlocksHistory,

    /// The validation service
    pub validation: Validation<Validator>,
}

impl<Validator: ValidateExtrinsic> SpaceJam<Validator> {
    /// Import a new block into the chain
    ///
    /// TODO: waiting for test data for block importing or authoring service.
    pub async fn import(&mut self, _block: Block) {}
}
