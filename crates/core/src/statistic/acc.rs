//! Accumulation statistics

use crate::{
    vm::{Accumulated, DeferredTransfer},
    Gas,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::collections::HashSet;

/// (I) Statistics about the accumulation process
///
/// Returns statistics about the accumulation process including:
/// - Total number of accumulated work reports
/// - Total gas used
/// - Number of services affected
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct AccumulationRecord {
    /// Number of work reports that were processed
    pub work_reports_processed: usize,
    /// Total gas used during accumulation
    pub total_gas_used: Gas,
    /// Number of services that were affected
    pub services_affected: usize,
    /// Number of accumulation commitments generated
    pub commitment_count: usize,
}

impl From<&Accumulated> for AccumulationRecord {
    fn from(accumulated: &Accumulated) -> Self {
        let affected_services: HashSet<_> = accumulated.context.accounts.keys().collect();

        AccumulationRecord {
            work_reports_processed: accumulated.accumulated,
            total_gas_used: accumulated.gas,
            services_affected: affected_services.len(),
            commitment_count: accumulated.pairings.len(),
        }
    }
}

/// (X) Statistics about deferred transfers
///
/// Returns statistics about deferred transfers including:
/// - Total number of transfers
/// - Total value transferred
/// - Number of unique source/destination services
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct TransferRecord {
    /// Total number of transfers
    pub transfer_count: usize,
    /// Total value transferred
    pub total_value: u64,
    /// Number of unique source services
    pub unique_source_services: usize,
    /// Number of unique destination services
    pub unique_dest_services: usize,
    /// Total gas allocated for transfers
    pub total_transfer_gas: Gas,
}

impl From<&[DeferredTransfer]> for TransferRecord {
    fn from(transfers: &[DeferredTransfer]) -> Self {
        let source_services: HashSet<_> = transfers.iter().map(|t| t.sender).collect();
        let dest_services: HashSet<_> = transfers.iter().map(|t| t.recipient).collect();
        let total_value: u64 = transfers.iter().map(|t| t.amount).sum();

        TransferRecord {
            transfer_count: transfers.len(),
            total_value,
            unique_source_services: source_services.len(),
            unique_dest_services: dest_services.len(),
            total_transfer_gas: transfers.iter().map(|t| t.gas_limit).sum(),
        }
    }
}
