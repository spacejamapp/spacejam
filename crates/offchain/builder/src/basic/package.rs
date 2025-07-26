//! Basic package validation and building logic

use crate::basic::item::ItemValidator;
use anyhow::{anyhow, Result};
use score::service::{Authorizer, RefineContext, WorkItem, WorkPackage};

/// Basic implementation of the Builder trait
pub struct Builder {
    /// Authorization token
    pub auth_token: Vec<u8>,
    /// Host service ID for authorization code
    pub auth_code_host: score::ServiceId,
    /// Authorization code hash
    pub auth_code_hash: score::OpaqueHash,
    /// Authorization configuration
    pub auth_config: Vec<u8>,
    /// Refine context
    pub context: RefineContext,
    /// Work items
    pub items: Vec<WorkItem>,
}

impl Builder {
    /// Calculate the total size of the work bundle
    fn calculate_bundle_size(&self, package: &WorkPackage) -> u32 {
        let mut size = 0u32;

        // Authorization token size
        size += package.authorization.len() as u32;

        // Authorizer params size
        size += package.authorizer.params.len() as u32;

        // Items payload and extrinsic sizes
        for item in &package.items {
            size += item.payload.len() as u32;

            // Import segments size (each segment is SEGMENT_SIZE)
            size += (item.import_segments.len() as u32) * score::SEGMENT_SIZE;

            // Extrinsic data sizes
            for extrinsic in &item.extrinsic {
                size += extrinsic.len;
            }
        }

        // Add overhead for work package structure itself (approximately 4KB)
        size += 4096;

        size
    }
}

impl crate::Builder for Builder {
    fn new_package(
        auth_token: Vec<u8>,
        auth_code_host: score::ServiceId,
        auth_code_hash: score::OpaqueHash,
        auth_config: Vec<u8>,
        context: RefineContext,
    ) -> Self {
        Self {
            auth_token,
            auth_code_host,
            auth_code_hash,
            auth_config,
            context,
            items: Vec::new(),
        }
    }

    fn add_item(&mut self, item: WorkItem) -> Result<&mut Self> {
        self.items.push(item);
        Ok(self)
    }

    fn build(self) -> Result<WorkPackage> {
        // Validate before building using ItemValidator
        ItemValidator::validate_gas_limits(&self.items)?;
        ItemValidator::validate_manifest_limits(&self.items)?;

        // Calculate bundle size before creating the package
        let mut bundle_size = 0u32;
        bundle_size += self.auth_token.len() as u32;
        bundle_size += self.auth_config.len() as u32;

        for item in &self.items {
            bundle_size += item.payload.len() as u32;
            bundle_size += (item.import_segments.len() as u32) * score::SEGMENT_SIZE;
            for extrinsic in &item.extrinsic {
                bundle_size += extrinsic.len;
            }
        }
        bundle_size += 4096; // Structure overhead

        if bundle_size > score::MAX_INPUT {
            return Err(anyhow!(
                "Work bundle size exceeded: {} > {}",
                bundle_size,
                score::MAX_INPUT
            ));
        }

        Ok(WorkPackage {
            authorization: self.auth_token,
            auth_code_host: self.auth_code_host,
            authorizer: Authorizer {
                code_hash: self.auth_code_hash,
                params: self.auth_config,
            },
            context: self.context,
            items: self.items,
        })
    }

    fn validate(package: &WorkPackage) -> Result<()> {
        // Validate gas limits using ItemValidator
        ItemValidator::validate_gas_limits(&package.items)?;

        // Validate manifest limits using ItemValidator
        ItemValidator::validate_manifest_limits(&package.items)?;

        // Validate bundle size
        let temp_builder = Builder {
            auth_token: Vec::new(),
            auth_code_host: 0,
            auth_code_hash: score::OpaqueHash::default(),
            auth_config: Vec::new(),
            context: RefineContext::default(),
            items: Vec::new(),
        };
        let bundle_size = temp_builder.calculate_bundle_size(package);
        if bundle_size > score::MAX_INPUT {
            return Err(anyhow!(
                "Work bundle size exceeded: {} > {}",
                bundle_size,
                score::MAX_INPUT
            ));
        }

        Ok(())
    }
}
