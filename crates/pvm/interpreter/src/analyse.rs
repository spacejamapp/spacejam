//! Analyse the program

use anyhow::Result;
use parser::{format::RIO, reader::Offset, Instruction};
use pvm::Program;
use std::collections::{BTreeMap, BTreeSet};

/// Analyse the program for function splitting
pub fn analyse(program: &Program) -> Result<()> {
    let blob = program.blob()?;

    let mut funcs = BTreeSet::new();
    // let mut ifuncs = BTreeSet::new();
    // let mut sjumps = Vec::new();
    let mut blocks = BTreeMap::new();
    let mut reader = blob.reader().with_position(0);
    while !reader.eof() {
        let block = reader.read_block()?;
        if let Some(last) = &block.last() {
            if let Instruction::LoadImmJump(RIO { off0, .. }) = last.value {
                funcs.insert(last.range.start as u64 + off0 as u64);
            }

            /* if let Some(target) = sjump(last) {
                // let target = last.range.start as i64 + offset as i64;
                ifuncs.insert(target as u64);
                println!(
                    "{}..{target}, distance={}",
                    last.range.start as u64,
                    (target - last.range.start as i64).abs()
                );
            } */

            /*   if let Some(sjump) = sjump(last) {
                sjumps.push(last.range.start as i32..sjump);
            } */
        }

        blocks.insert(reader.position as u64, block);
    }

    for target in &blob.jump_table {
        let Some(block) = blocks.get(target) else {
            continue;
        };
        /* let Some(last) = block.last() else {
            continue;
        }; */

        println!("djump: {} -> {:?}", target, block);
    }

    tracing::debug!("djump({}): {:?}", blob.jump_table.len(), blob.jump_table);
    // tracing::debug!("sjumps(len={}): {:?}", sjumps.len(), sjumps);
    tracing::debug!("funcs(len={}): {:?}", funcs.len(), funcs);
    tracing::debug!("blocks(len={})", blocks.len());
    Ok(())
}

fn _sjump(last: &Offset<Instruction>) -> Option<i64> {
    match last.value {
        Instruction::Jump(offset) => Some(last.range.start as i64 + offset.off0 as i64),
        Instruction::BranchEq(offset) => Some(last.range.start as i64 + offset.off0 as i64),
        Instruction::BranchNe(offset) => Some(last.range.start as i64 + offset.off0 as i64),
        Instruction::BranchGeU(offset) => Some(last.range.start as i64 + offset.off0 as i64),
        Instruction::BranchGeS(offset) => Some(last.range.start as i64 + offset.off0 as i64),
        Instruction::BranchLtU(offset) => Some(last.range.start as i64 + offset.off0 as i64),
        Instruction::BranchLtS(offset) => Some(last.range.start as i64 + offset.off0 as i64),
        Instruction::BranchEqImm(offset) => Some(last.range.start as i64 + offset.off0 as i64),
        Instruction::BranchNeImm(offset) => Some(last.range.start as i64 + offset.off0 as i64),
        Instruction::BranchGeUImm(offset) => Some(last.range.start as i64 + offset.off0 as i64),
        Instruction::BranchGeSImm(offset) => Some(last.range.start as i64 + offset.off0 as i64),
        _ => None,
    }
}
