//! Analyse the program

use std::collections::BTreeSet;

use anyhow::Result;
use parser::ir::IR;
use pvm::Program;

/// Analyse the program for function splitting
pub fn analyse(program: &Program) -> Result<()> {
    let blob = program.blob()?;
    let mut ir = IR::default();
    ir.parse(&blob)?;

    println!("functions: {}", ir.funcs.len());
    let table2 = blob.jump_table.clone().into_iter().collect::<BTreeSet<_>>();
    println!(
        "jump table length: {:?}, expected: {}, deduplicated: {}",
        ir.funcs
            .values()
            .map(|func| func.jump.len())
            .collect::<Vec<_>>()
            .iter()
            .sum::<usize>(),
        blob.jump_table.len(),
        table2.len()
    );
    let _ = ir.verify();
    Ok(())
}
