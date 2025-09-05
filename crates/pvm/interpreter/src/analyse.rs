//! Analyse the program

use anyhow::Result;
use parser::Instruction;
use pvm::Program;

/// Analyse the program for function splitting
pub fn analyse(program: &Program) -> Result<()> {
    let blob = program.blob()?;
    tracing::debug!(
        "jump table(len={}): {:?}",
        blob.jump_table.len(),
        blob.jump_table
    );

    let mut reader = blob.reader().with_position(0);
    let mut count = 0;
    while !reader.eof() {
        let block = reader.read_block()?;
        if let Some(last) = block.last() {
            if !matches!(last.value, Instruction::Fallthrough | Instruction::Trap)
                && blob.jump_table.contains(&(last.range.end as u64))
            {
                tracing::debug!(
                    "block at pc={} contains {:?} blocks, end with {:?}",
                    reader.position,
                    block.len(),
                    last.value
                );
            }
        }
        count += 1;
    }
    tracing::debug!("total blocks: {}", count);

    Ok(())
}
