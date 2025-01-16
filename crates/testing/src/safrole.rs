//! Safrole vector tests

use score::{
    block::header::{EpochMark, EpochMarkJson, TicketsMark},
    extrinsic::ticket::{TicketBodyJson, TicketEnvelopeJson, TicketsExtrinsic},
    OpaqueHash,
};
use serde::{Deserialize, Serialize};
use spacejson::{Json, ResultJson};
use ticket::{Error, State, StateJson};

/// Test input.
#[derive(Deserialize, Serialize, Json, Debug)]
pub struct Input {
    pub slot: u32,
    #[json(hex)]
    pub entropy: OpaqueHash,
    #[json(Vec<TicketEnvelopeJson>)]
    pub extrinsic: TicketsExtrinsic,
}

/// Test input.
#[derive(Deserialize, Serialize, Json, Debug)]
pub struct TestInput {
    #[json(nested)]
    pub input: Input,
    #[json(nested)]
    pub pre_state: State,
}

/// Test output.
#[derive(Deserialize, Serialize, Json, Debug)]
pub struct TestOutput {
    #[json(ResultJson<MarkersJson, Error>)]
    pub output: std::result::Result<Markers, Error>,
    #[json(nested)]
    pub post_state: State,
}

crate::impl_tests! {
    safrole,
    @scale
    enact_epoch_change_with_no_tickets_1,
    enact_epoch_change_with_no_tickets_2,
    enact_epoch_change_with_no_tickets_3,
    enact_epoch_change_with_no_tickets_4,
    enact_epoch_change_with_padding_1,
    publish_tickets_no_mark_1,
    publish_tickets_no_mark_2,
    publish_tickets_no_mark_3,
    publish_tickets_no_mark_4,
    publish_tickets_no_mark_5,
    publish_tickets_no_mark_6,
    publish_tickets_no_mark_7,
    publish_tickets_no_mark_8,
    publish_tickets_no_mark_9,
    publish_tickets_with_mark_1,
    publish_tickets_with_mark_2,
    publish_tickets_with_mark_3,
    publish_tickets_with_mark_4,
    publish_tickets_with_mark_5,
    skip_epoch_tail_1,
    skip_epochs_1
}

/// Represents the Output marks
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Json)]
pub struct Markers {
    /// New epoch marker
    #[json(nested)]
    pub epoch_mark: Option<EpochMark>,
    /// New tickets marker
    #[json(Option<Vec<TicketBodyJson>>)]
    pub tickets_mark: Option<TicketsMark>,
}
