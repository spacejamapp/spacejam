//! IR verifier

use crate::ir::IR;
use anyhow::Result;

impl IR {
    /// Verify the IR
    pub fn verify(&self, table: &[u64]) -> Result<()> {
        // check all jump table entries are in the IR
        let jsize = self
            .funcs
            .values()
            .map(|func| func.jump.len())
            .collect::<Vec<_>>()
            .iter()
            .sum::<usize>();
        if jsize != table.len() {
            anyhow::bail!("jump table length mismatch: {jsize} != {}", table.len());
        }

        // check all jumps are resolved
        //
        // 1. jump to local function
        // 2. jump to a function
        for (func_pc, func) in &self.funcs {
            for block_pc in &func.blocks {
                let Some(block) = self.blocks.get(block_pc) else {
                    anyhow::bail!("block {func_pc}:{block_pc} not found");
                };

                let reachable = block.reachable();
                if self.funcs.contains_key(&reachable) {
                    continue;
                }

                if func.range.contains(&reachable) || func.range.end == reachable {
                    continue;
                }

                if func.jump.values().any(|pc| *pc == reachable) {
                    continue;
                }

                eprintln!("{func_pc}:{block_pc} jump to unresolved program counter: {reachable}");
            }
        }

        Ok(())
    }
}
