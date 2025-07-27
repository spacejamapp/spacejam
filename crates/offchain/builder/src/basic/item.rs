//! Basic item builder implementation

use crate::ItemBuilder;
use anyhow::{anyhow, Result};
use score::service::{ExtrinsicSpec, ImportSpec, WorkItem};

/// Basic implementation of ItemBuilder
#[derive(Debug, Default)]
pub struct Builder {
    refine_gas_limit: score::Gas,
    accumulate_gas_limit: score::Gas,
    imports: Vec<ImportSpec>,
    extrinsics: Vec<ExtrinsicSpec>,
    export_count: u16,
}

impl ItemBuilder for Builder {
    fn refine_gas_limit(mut self, gas: score::Gas) -> Self {
        self.refine_gas_limit = gas;
        self
    }

    fn accumulate_gas_limit(mut self, gas: score::Gas) -> Self {
        self.accumulate_gas_limit = gas;
        self
    }

    fn add_import(mut self, tree_root: score::OpaqueHash, index: u16) -> Result<Self> {
        if self.imports.len() >= score::MAX_IMPORTS as usize {
            return Err(anyhow!(
                "Import count limit exceeded: {} >= {}",
                self.imports.len(),
                score::MAX_IMPORTS
            ));
        }

        self.imports.push(ImportSpec { tree_root, index });
        Ok(self)
    }

    fn add_extrinsic(mut self, extrinsic: ExtrinsicSpec) -> Result<Self> {
        if self.extrinsics.len() >= score::MAX_EXTRINSICS as usize {
            return Err(anyhow!(
                "Extrinsic count limit exceeded: {} >= {}",
                self.extrinsics.len(),
                score::MAX_EXTRINSICS
            ));
        }

        self.extrinsics.push(extrinsic);
        Ok(self)
    }

    fn export_count(mut self, count: u16) -> Self {
        self.export_count = count;
        self
    }

    fn build(self) -> Result<WorkItem> {
        // Note: This creates a WorkItem with empty import_segments and extrinsic vectors
        // because the core ImportSpec/ExtrinsicSpec types are not exported from spacejam-core.
        // Use extrinsic_commitments() and extrinsic_data() for proper work package building.

        Ok(WorkItem {
            service: 0,
            code_hash: score::OpaqueHash::default(),
            payload: Vec::new(),
            refine_gas_limit: self.refine_gas_limit,
            accumulate_gas_limit: self.accumulate_gas_limit,
            import_segments: Vec::new(),
            extrinsic: Vec::new(),
            export_count: self.export_count,
        })
    }
}
