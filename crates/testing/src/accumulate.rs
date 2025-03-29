//! Accumulate tests

use score::{
    service::{
        AccumulatedQueue, Privileges, PrivilegesJson, ReadyQueue, ReadyRecordJson, ServiceItem,
        ServiceItemJson, WorkReport, WorkReportJson,
    },
    Entropy, OpaqueHash, TimeSlot,
};
use serde::{Deserialize, Serialize};
use spacejson::{Json, ResultJson};

/// Accumulate test
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct Test {
    /// The input
    #[json(nested)]
    pub input: TestInput,

    /// The output
    #[json(ResultJson<String, ()>)]
    pub output: Result<OpaqueHash, ()>,
}

/// Test input
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct Input {
    /// The time slot from the block header
    pub slot: TimeSlot,

    /// The reports
    #[json(nested)]
    pub reports: Vec<WorkReport>,
}

/// Test input
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct TestInput {
    /// The input
    #[json(nested)]
    pub input: Input,

    /// The pre-state
    #[json(nested)]
    pub pre_state: State,
}

/// Test output
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct TestOutput {
    /// The post-state
    #[json(nested)]
    pub post_state: State,

    /// The output
    #[json(ResultJson<String, ()>)]
    pub output: Result<OpaqueHash, ()>,
}

/// State for the accumulation
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct State {
    /// The time slot
    pub slot: TimeSlot,

    /// The current entropy
    #[json(hex)]
    pub entropy: Entropy,

    /// The ready queue
    #[json(Vec<Vec<ReadyRecordJson>>)]
    pub ready_queue: ReadyQueue,

    /// The accumulated reports
    #[json(Vec<Vec<String>>)]
    pub accumulated: AccumulatedQueue,

    /// The privileges
    #[json(nested)]
    pub privileges: Privileges,

    /// The accounts
    #[json(nested)]
    pub accounts: Vec<ServiceItem>,
}

crate::impl_tests! {
    accumulate,
    @scale
    accumulate_ready_queued_reports_1,
    enqueue_and_unlock_chain_wraps_1,
    enqueue_and_unlock_chain_wraps_2,
    enqueue_and_unlock_chain_wraps_3,
    enqueue_and_unlock_chain_wraps_4,
    enqueue_and_unlock_chain_wraps_5,
    enqueue_and_unlock_chain_1,
    enqueue_and_unlock_chain_2,
    enqueue_and_unlock_chain_3,
    enqueue_and_unlock_chain_4,
    enqueue_and_unlock_simple_1,
    enqueue_and_unlock_simple_2,
    enqueue_and_unlock_with_sr_lookup_1,
    enqueue_and_unlock_with_sr_lookup_2,
    enqueue_self_referential_1,
    enqueue_self_referential_2,
    enqueue_self_referential_3,
    enqueue_self_referential_4,
    no_available_reports_1,
    process_one_immediate_report_1,
    queues_are_shifted_1,
    queues_are_shifted_2,
    ready_queue_editing_1,
    ready_queue_editing_2,
    ready_queue_editing_3

}
