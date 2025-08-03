//! Guarantor abstraction

use std::collections::BTreeMap;

use crate::{bundle::WorkPackageBundle, d3l::Shard, DataLake};
use anyhow::Result;
use score::{
    extrinsic::{ReportGuarantee, ValidatorSignature},
    service::{WorkPackage, WorkReport},
    Accounts, CoreIndex, OpaqueHash, TimeSlot,
};

/// Guarantor abstraction
#[allow(async_fn_in_trait)]
pub trait Guarantor: DataLake {
    /// On receiving a work package submission (CE133)
    ///
    /// Builder -> Guarantor
    async fn compute<A: Accounts>(
        &self,
        _core_index: CoreIndex,
        _extrinsic: Vec<Vec<u8>>,
        _package: &WorkPackage,
        _accounts: &mut A,
    ) -> Result<(WorkReport, WorkPackageBundle)> {
        todo!()
    }

    /// Validate a work package bundle (CE134)
    ///
    /// Guarantor -> Guarantor
    async fn validate<A: Accounts>(
        &self,
        _bundle: &WorkPackageBundle,
        _core_index: CoreIndex,
        _segment_roots: BTreeMap<OpaqueHash, OpaqueHash>,
        _accounts: &mut A,
    ) -> Result<(WorkReport, ValidatorSignature)> {
        todo!()
    }

    /// Create guaranteed work report (CE135)
    ///
    /// Guarantor -> Validator
    async fn guarantee(
        &self,
        _report: &WorkReport,
        _slot: TimeSlot,
        _singatures: Vec<ValidatorSignature>,
    ) -> Result<ReportGuarantee> {
        todo!()
    }

    /// On shard requests (CE137)
    async fn shard(&self, _erasure_root: OpaqueHash, _shard_index: u16) -> Result<Shard> {
        todo!()
    }
}
