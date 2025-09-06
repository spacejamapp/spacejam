//! Analyse the program

use anyhow::Result;
use parser::{
    format::{O, RIO},
    reader::Offset,
    Instruction,
};
use pvm::Program;
use std::collections::{BTreeMap, BTreeSet};

/// Analyse the program for function splitting
pub fn analyse(program: &Program) -> Result<()> {
    let blob = program.blob()?;

    let mut funcs = BTreeSet::new();
    let mut blocks = BTreeMap::new();
    let mut reader = blob.reader().with_position(0);
    while !reader.eof() {
        let block = reader.read_block()?;
        if let Some(last) = &block.last() {
            if let Instruction::LoadImmJump(RIO { off0, .. }) = last.value {
                funcs.insert(last.range.start as u64 + off0 as u64);
            }

            if last.range.start < 15 {
                if let Instruction::Jump(O { off0, .. }) = last.value {
                    funcs.insert((last.range.start as i64 + off0 as i64) as u64);
                }
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

    let funcs = funcs.into_iter().collect::<Vec<_>>();
    let rfuncs = funcs
        .iter()
        .collect::<Vec<_>>()
        .windows(2)
        .map(|w| w[0]..w[1])
        .collect::<Vec<_>>();
    tracing::debug!("blocks(len={})", blocks.len());
    tracing::debug!("rfuncs(len={}): {:?}", rfuncs.len(), rfuncs);

    // undiscovered functions
    let mut ufuncs = BTreeSet::new();
    let mut cfun = 0;
    println!("-> function 0 {:?}:", rfuncs[cfun]);
    for target in &blob.jump_table {
        let Some(block) = blocks.get(target) else {
            continue;
        };

        let Some(start) = block.first() else {
            continue;
        };

        let Some(last) = block.last() else {
            continue;
        };

        let Some(reachable) = reachable(last) else {
            continue;
        };

        while start.range.start > funcs[cfun + 1] as usize {
            cfun += 1;
            println!("-> function {cfun} ({:?}):", rfuncs[cfun]);
        }

        if !funcs.contains(&(reachable as u64))
            && !blob.jump_table.contains(&(reachable as u64))
            && (reachable as usize > (*rfuncs[cfun].end) as usize
                || (reachable as usize) < (*rfuncs[cfun].start) as usize)
        {
            ufuncs.insert(reachable as u64);
            println!("logic broken, unhandled block discovered, could be a new function!");
        }

        // Check if the target block is within the local function.
        //
        // 1. if the target block contains only one instruction.
        // 2. if the target block contains the usage of ra
        println!("    djump: {} -> {reachable}: {:?}", target, block);
    }

    tracing::debug!("ufuncs(len={}): {:?}", ufuncs.len(), ufuncs);
    tracing::debug!("djump({}): {:?}", blob.jump_table.len(), blob.jump_table);
    Ok(())
}

fn reachable(last: &Offset<Instruction>) -> Option<i64> {
    match last.value {
        Instruction::Trap => Some(last.range.end as i64),
        Instruction::Fallthrough => Some(last.range.end as i64),
        Instruction::LoadImmJump(offset) => Some(last.range.start as i64 + offset.off0 as i64),
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
