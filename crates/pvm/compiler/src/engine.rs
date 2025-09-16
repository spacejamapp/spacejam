//! SpaceJam PVM compiler engine

use anyhow::Result;
use cranelift::{codegen::settings, native, prelude::Configurable};
use cranelift_codegen::isa::OwnedTargetIsa;

/// Sort of config for the compilation
pub struct Engine;

impl Engine {
    /// Maximum number of compilation speed
    pub fn compilation() -> Result<OwnedTargetIsa> {
        let mut builder = settings::builder();
        builder.set("opt_level", "none")?;
        builder.set("enable_verifier", "false")?;
        builder.set("enable_alias_analysis", "false")?;
        builder.set("regalloc_checker", "false")?;
        builder.set("regalloc_verbose_logs", "false")?;
        builder.set("enable_incremental_compilation_cache_checks", "false")?;
        builder.set("unwind_info", "false")?;
        builder.set("machine_code_cfg_info", "false")?;
        builder.set("enable_pcc", "false")?;

        // Create the ISA builder and finish it with the flags
        let isa_builder = native::builder().map_err(|e| anyhow::anyhow!("{}", e))?;
        let isa = isa_builder.finish(settings::Flags::new(builder))?;
        Ok(isa)
    }

    /// Maximum number of execution speed
    pub fn speed() -> Result<OwnedTargetIsa> {
        let mut builder = settings::builder();
        builder.set("opt_level", "speed")?;
        builder.set("enable_verifier", "false")?;
        builder.set("enable_alias_analysis", "false")?;
        builder.set("regalloc_checker", "false")?;
        builder.set("regalloc_verbose_logs", "false")?;
        builder.set("enable_incremental_compilation_cache_checks", "false")?;
        builder.set("unwind_info", "false")?;
        builder.set("machine_code_cfg_info", "false")?;
        builder.set("enable_pcc", "false")?;

        // Create the ISA builder and finish it with the flags
        let isa_builder = native::builder().map_err(|e| anyhow::anyhow!("{}", e))?;
        let isa = isa_builder.finish(settings::Flags::new(builder))?;
        Ok(isa)
    }
}
