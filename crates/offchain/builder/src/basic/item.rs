//! Basic item validation logic

use anyhow::{anyhow, Result};
use score::service::WorkItem;

/// Item-related validation methods for BasicBuilder
pub struct ItemValidator;

impl ItemValidator {
    /// Validate gas limits according to Gray Paper constraints
    pub fn validate_gas_limits(items: &[WorkItem]) -> Result<()> {
        let mut total_refine_gas = 0u64;
        let mut total_accumulate_gas = 0u64;
        
        for item in items {
            total_refine_gas += item.refine_gas_limit;
            total_accumulate_gas += item.accumulate_gas_limit;
        }
        
        // Check against per-core gas limits
        let max_refine_gas = score::GAS_REFINE;
        let max_accumulate_gas = score::GAS_ACC;
        
        if total_refine_gas > max_refine_gas {
            return Err(anyhow!("Gas limit exceeded: {} > {}", total_refine_gas, max_refine_gas));
        }
        
        if total_accumulate_gas > max_accumulate_gas {
            return Err(anyhow!("Gas limit exceeded: {} > {}", total_accumulate_gas, max_accumulate_gas));
        }
        
        Ok(())
    }
    
    /// Validate manifest limits (imports, exports, extrinsics)
    pub fn validate_manifest_limits(items: &[WorkItem]) -> Result<()> {
        let mut total_exports = 0u32;
        let mut total_imports = 0u32;
        let mut total_extrinsics = 0u16;
        
        for item in items {
            total_exports += item.export_count as u32;
            total_imports += item.import_segments.len() as u32;
            total_extrinsics += item.extrinsic.len() as u16;
        }
        
        if total_exports > score::MAX_EXPORTS {
            return Err(anyhow!("Export count exceeded: {} > {}", total_exports, score::MAX_EXPORTS));
        }
        
        if total_imports > score::MAX_IMPORTS {
            return Err(anyhow!("Import count exceeded: {} > {}", total_imports, score::MAX_IMPORTS));
        }
        
        if total_extrinsics > score::MAX_EXTRINSICS {
            return Err(anyhow!("Extrinsic count exceeded: {} > {}", total_extrinsics, score::MAX_EXTRINSICS));
        }
        
        Ok(())
    }
}