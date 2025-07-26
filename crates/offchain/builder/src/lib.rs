//! Spacejam work package builder
//!
//! This module provides the work package builder infrastructure for SpaceJam,
//! allowing the construction of work packages according to the Gray Paper specification.
//!
//! Note: Currently the builder uses local copies of ImportSpec and ExtrinsicSpec types
//! because the core types are not publicly exposed. This should be addressed in a future
//! update to spacejam-core.

pub use builder::{
    Builder, DefaultWorkItemBuilder, ExtrinsicSpec, ImportSpec, ItemBuilder, WorkPackageBuilder,
};

pub mod basic;
pub mod builder;
