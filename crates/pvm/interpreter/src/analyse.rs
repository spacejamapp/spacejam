//! Analyse the program

use anyhow::Result;
use parser::ir::IR;
use pvm::Program;

/// Analyse the program for function splitting
pub fn analyse(program: &Program) -> Result<()> {
    let blob = program.blob()?;
    let mut ir = IR::default();
    ir.parse(&blob)?;
    println!("{:?}", ir);
    let _ = ir.verify();
    Ok(())
}
