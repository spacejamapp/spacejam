//! Analyse the program

use anyhow::Result;
use parser::ir::IR;
use pvm::Program;

/// Analyse the program for function splitting
pub fn analyse(program: &Program) -> Result<()> {
    let blob = program.blob()?;
    let mut ir = IR::default();
    ir.parse(&blob)?;

    println!("total functions: {}", ir.funcs.len());
    for (entry, func) in &ir.funcs {
        println!(
            "function:{entry}({:?}): blocks={} jumps({})={:?}",
            func.range,
            func.blocks.len(),
            func.jump.len(),
            func.jump.values(),
        );
    }

    println!("exports: {:?}", ir.exports);
    let accumulate = ir.export(5).expect("accumulate not found");
    println!(
        "accumulate({:?}): functions={:?}",
        accumulate.funcs.len(),
        accumulate.funcs.keys().collect::<Vec<_>>(),
    );

    if let Err(e) = ir.verify(&blob.jump_table) {
        println!("{e}");
    }
    Ok(())
}
