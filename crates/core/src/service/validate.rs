//! work package validation which is for statistic usages

use crate::service::WorkPackage;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents a detailed view of a work package.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct PackageValidation {
    /// The total exports
    pub total_exports: u32,

    /// The total imports
    pub total_imports: u32,

    /// The total extrinsics
    pub total_extrinsics: u32,

    /// The total refine gas
    pub total_refine_gas: u64,

    /// The total accumulate gas
    pub total_accumulate_gas: u64,

    /// The bundle size
    pub bundle_size: u64,
}

impl PackageValidation {
    /// Create a new package validation
    pub fn new(package: &WorkPackage) -> Self {
        let mut total_exports = 0u32;
        let mut total_imports = 0u32;
        let mut total_extrinsics = 0u32;
        let mut total_refine_gas = 0u64;
        let mut total_accumulate_gas = 0u64;
        let mut bundle_size = 0u64;
        bundle_size += package.authorization.len() as u64;
        bundle_size += package.config.len() as u64;
        for item in &package.items {
            // Count limits
            total_exports += item.export_count as u32;
            total_imports += item.import_segments.len() as u32;
            total_extrinsics += item.extrinsic.len() as u32;

            // Gas limits
            total_refine_gas += item.refine_gas_limit;
            total_accumulate_gas += item.accumulate_gas_limit;

            // Bundle size calculation: S(w) = |w_payload| + |w_importsegments|·W_G + ∑_{(h,l) ∈ w_extrinsics} l
            bundle_size += item.payload.len() as u64;
            bundle_size += (item.import_segments.len() as u64) * (crate::SEGMENT_SIZE as u64);
            bundle_size += item.extrinsic.iter().map(|ext| ext.len as u64).sum::<u64>();
        }

        Self {
            total_exports,
            total_imports,
            total_extrinsics,
            total_refine_gas,
            total_accumulate_gas,
            bundle_size,
        }
    }

    /// Validate the package
    pub fn validate(&self) -> Result<()> {
        // Check export count limit (W_X = 3072)
        if self.total_exports > crate::MAX_EXPORTS {
            anyhow::bail!(
                "total export count {} exceeds maximum {}",
                self.total_exports,
                crate::MAX_EXPORTS
            );
        }

        // Check import count limit (W_M = 3072)
        if self.total_imports > crate::MAX_IMPORTS {
            anyhow::bail!(
                "total import count {} exceeds maximum {}",
                self.total_imports,
                crate::MAX_IMPORTS
            );
        }

        // Check extrinsic count limit (T = 128)
        if self.total_extrinsics > crate::MAX_EXTRINSICS as u32 {
            anyhow::bail!(
                "total extrinsic count {} exceeds maximum {}",
                self.total_extrinsics,
                crate::MAX_EXTRINSICS
            );
        }

        // Check work bundle size limit (W_B = 12MB)
        if self.bundle_size > crate::MAX_INPUT as u64 {
            anyhow::bail!(
                "work bundle size {} exceeds maximum {} bytes",
                self.bundle_size,
                crate::MAX_INPUT
            );
        }

        // Check gas limits
        if self.total_refine_gas > crate::GAS_REFINE {
            anyhow::bail!(
                "total refine gas {} exceeds maximum {}",
                self.total_refine_gas,
                crate::GAS_REFINE
            );
        }

        if self.total_accumulate_gas > crate::GAS_ALL_ACC {
            anyhow::bail!(
                "total accumulate gas {} exceeds maximum {}",
                self.total_accumulate_gas,
                crate::GAS_ALL_ACC
            );
        }

        Ok(())
    }
}
