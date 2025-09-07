//! Control flow resolver

use crate::ir::{Control, FunctionRef, IR};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};

impl IR {
    /// Resolve the control flow
    ///
    /// 1. resolve all knwon jumps
    /// 2. resolve multi sources jumps
    pub fn resolve(&mut self, mut ujumps: BTreeMap<u64, Vec<u64>>) -> Result<()> {
        ujumps.retain(|target, _src| !self.funcs.contains_key(target));

        let mut size = ujumps.len();
        loop {
            self.resolve_internal(&mut ujumps)?;
            let rest = ujumps.len();
            if rest == size {
                break;
            }
            size = rest;
        }

        Ok(())
    }

    /// resolve internal jumps
    pub fn resolve_internal(&mut self, ujumps: &mut BTreeMap<u64, Vec<u64>>) -> Result<()> {
        let jumps = ujumps.clone();
        for (to, from) in jumps {
            let source = from
                .iter()
                .filter_map(|src| {
                    self.funcs
                        .iter()
                        .find(|(_, f)| f.range.contains(src))
                        .map(|(_, f)| f.range.start)
                })
                .collect::<BTreeSet<_>>();

            // split if there is only one source
            if source.len() == 1 {
                let Some(func) = source
                    .first()
                    .and_then(|pc| self.funcs.get(pc).map(|b| b.range.clone()))
                else {
                    anyhow::bail!("failed to find source function for block {to}");
                };

                if func.contains(&to) || func.end == to {
                    ujumps.remove(&to);
                    continue;
                }
            }

            ujumps.remove(&to);
            ujumps.extend(self.split(to)?);
        }
        Ok(())
    }

    /// Split out functions via the given entrypoint
    fn split(&mut self, entry: u64) -> Result<BTreeMap<u64, Vec<u64>>> {
        let (_, func) = self
            .funcs
            .iter_mut()
            .find(|(_, f)| f.range.contains(&entry))
            .ok_or(anyhow::anyhow!(
                "could not locate function for block {entry}"
            ))?;

        // update the range of the functions
        let mut next = FunctionRef::new(entry);
        next.range.end = func.range.end;
        func.range.end = entry;

        // scale the blocks and the jump table
        (func.jump, next.jump) = func.jump.iter().partition(|(_, pc)| **pc < entry);
        (func.blocks, next.blocks) = func.blocks.iter().partition(|&pc| *pc < entry);
        let prev = func.clone();
        self.funcs.insert(entry, next.clone());

        // create new unresolved jumps
        let mut ujumps = BTreeMap::new();
        for func in [prev, next] {
            for block in &func.blocks {
                let Some(block) = self.blocks.get(block) else {
                    anyhow::bail!("{}:{block} not found", func.range.start);
                };

                if let Control::Jump(target) = block.control {
                    if func.range.contains(&target)
                        || func.range.end == target
                        || self.funcs.contains_key(&target)
                    {
                        continue;
                    }

                    ujumps
                        .entry(target)
                        .or_insert(vec![])
                        .push(block.range.start);
                }
            }
        }

        Ok(ujumps)
    }
}
