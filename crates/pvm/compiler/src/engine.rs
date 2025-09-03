//! SpaceJam PVM compiler engine

use anyhow::Result;
use cranelift::prelude::Configurable;
use cranelift_codegen::settings;
use cranelift_jit::JITBuilder;
use cranelift_module::default_libcall_names;

/// Maximum number of compilation speed
pub fn compilation() -> Result<JITBuilder> {
    let mut builder = settings::builder();
    builder.set("opt_level", "none")?;
    builder.set("enable_verifier", "true")?;
    builder.set("enable_alias_analysis", "false")?;
    builder.set("regalloc_checker", "false")?;
    builder.set("regalloc_verbose_logs", "false")?;
    builder.set("enable_incremental_compilation_cache_checks", "false")?;
    builder.set("unwind_info", "false")?;
    builder.set("machine_code_cfg_info", "false")?;
    builder.set("enable_pcc", "false")?;

    // Create the ISA builder and finish it with the flags
    let isa_builder = cranelift_native::builder().map_err(|e| anyhow::anyhow!("{}", e))?;
    let isa = isa_builder.finish(settings::Flags::new(builder))?;
    Ok(JITBuilder::with_isa(isa, default_libcall_names()))
}
