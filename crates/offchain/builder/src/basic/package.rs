//! Basic package validation and building logic

use crate::{Builder, WorkPackageBuilder};
use anyhow::{anyhow, Result};
use score::service::{Authorizer, RefineContext, WorkPackage};
use crate::basic::item::ItemValidator;

/// Basic implementation of the Builder trait
pub struct BasicBuilder;

impl BasicBuilder {
    /// Create a new instance of BasicBuilder
    pub fn new() -> Self {
        BasicBuilder
    }
    
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

impl Default for BasicBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder for BasicBuilder {
    fn new_package(
        &self,
        auth_token: Vec<u8>,
        auth_code_host: score::ServiceId,
        auth_code_hash: score::OpaqueHash,
        auth_config: Vec<u8>,
        context: RefineContext,
    ) -> Result<WorkPackageBuilder> {
        Ok(WorkPackageBuilder {
            auth_token,
            auth_code_host,
            auth_code_hash,
            auth_config,
            context,
            items: Vec::new(),
        })
    }
    
    fn build(&self, builder: WorkPackageBuilder) -> Result<WorkPackage> {
        // Validate before building using ItemValidator
        ItemValidator::validate_gas_limits(&builder.items)?;
        ItemValidator::validate_manifest_limits(&builder.items)?;
        
        let package = WorkPackage {
            authorization: builder.auth_token,
            auth_code_host: builder.auth_code_host,
            authorizer: Authorizer {
                code_hash: builder.auth_code_hash,
                params: builder.auth_config,
            },
            context: builder.context,
            items: builder.items,
        };
        
        // Validate bundle size
        let bundle_size = self.calculate_bundle_size(&package);
        if bundle_size > score::MAX_INPUT {
            return Err(anyhow!("Work bundle size exceeded: {} > {}", bundle_size, score::MAX_INPUT));
        }
        
        Ok(package)
    }
    
    fn validate(&self, package: &WorkPackage) -> Result<()> {
        // Validate gas limits using ItemValidator
        ItemValidator::validate_gas_limits(&package.items)?;
        
        // Validate manifest limits using ItemValidator
        ItemValidator::validate_manifest_limits(&package.items)?;
        
        // Validate bundle size
        let bundle_size = self.calculate_bundle_size(package);
        if bundle_size > score::MAX_INPUT {
            return Err(anyhow!("Work bundle size exceeded: {} > {}", bundle_size, score::MAX_INPUT));
        }
        
        // Additional validation can be added here
        
        Ok(())
    }
}