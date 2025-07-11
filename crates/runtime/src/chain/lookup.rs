//! Lookup the chain.

use crate::{chain::Fork, storage::SyncStorage};
use anyhow::Result;
use score::{Block, OpaqueHash};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// The direction of the lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[repr(u8)]
pub enum Direction {
    /// Fetch the block in ascending order.
    #[default]
    Ascending,

    /// Fetch the block in descending order.
    Descending,
}

/// Lookup the chain.
pub trait Lookup: Sized {
    /// Fetch the block in the given direction.
    fn fetch(
        &self,
        from: OpaqueHash,
        direction: Direction,
        maximum: usize,
    ) -> Result<impl Iterator<Item = Block>> {
        LookupIter::new(self, from, direction, maximum)
    }

    /// Get the block of the given hash.
    fn block(&self, hash: OpaqueHash) -> Result<Block>;

    /// Get the parent of the given block.
    fn parent(&self, block: OpaqueHash) -> Result<OpaqueHash>;

    /// Get the descendant of the given block.
    fn descendant(&self, block: OpaqueHash) -> Result<OpaqueHash>;
}

/// The iterator of the chain lookup.
pub struct LookupIter<'s, S: Lookup> {
    /// The storage of the chain.
    lookup: &'s S,

    /// The blocks of the lookup.
    blocks: VecDeque<OpaqueHash>,
}

impl<'s, S: Lookup> LookupIter<'s, S> {
    /// Create a new chain lookup iterator.
    pub fn new(
        lookup: &'s S,
        from: OpaqueHash,
        direction: Direction,
        count: usize,
    ) -> Result<Self> {
        if count == 0 {
            return Ok(Self {
                lookup,
                blocks: VecDeque::new(),
            });
        }

        let mut blocks = Vec::new();
        let mut current = from;
        match direction {
            Direction::Ascending => {
                while blocks.len() < count {
                    match lookup.descendant(current) {
                        Ok(next) => {
                            blocks.push(next);
                            current = next;
                        }
                        Err(_) => break,
                    }
                }
            }
            Direction::Descending => {
                blocks.push(current);
                while blocks.len() < count {
                    match lookup.parent(current) {
                        Ok(parent) => {
                            current = parent;
                            blocks.push(current);
                        }
                        Err(_) => break,
                    }
                }
            }
        };

        Ok(Self {
            lookup,
            blocks: blocks.into(),
        })
    }
}

impl<'s, S: Lookup> Iterator for LookupIter<'s, S> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        self.blocks
            .pop_front()
            .and_then(|hash| self.lookup.block(hash).ok())
    }
}

impl<S: SyncStorage> Lookup for S {
    fn block(&self, hash: OpaqueHash) -> Result<Block> {
        self.block(&hash)
    }

    fn descendant(&self, block: OpaqueHash) -> Result<OpaqueHash> {
        self.descendant(&block)
    }

    fn parent(&self, block: OpaqueHash) -> Result<OpaqueHash> {
        self.parent(&block)
    }
}

impl<S: SyncStorage> Lookup for Fork<S> {
    fn block(&self, hash: OpaqueHash) -> Result<Block> {
        if let Some(block) = self
            .chain
            .iter()
            .find_map(|head| {
                if head.hash == hash {
                    Some(head.clone())
                } else {
                    None
                }
            })
            .and_then(|head| {
                self.blocks
                    .get(&head.slot)
                    .map(|(block, _diff)| block.clone())
            })
        {
            Ok(block)
        } else {
            Lookup::block(&*self.state, hash)
        }
    }

    fn descendant(&self, block: OpaqueHash) -> Result<OpaqueHash> {
        if let Some(hash) = self
            .chain
            .iter()
            .position(|h| h.hash == block)
            .and_then(|pos| pos.checked_add(1))
            .and_then(|pos| self.chain.iter().nth(pos).map(|h| h.hash))
        {
            Ok(hash)
        } else {
            Lookup::descendant(&*self.state, block)
        }
    }

    fn parent(&self, block: OpaqueHash) -> Result<OpaqueHash> {
        if let Some(hash) = self
            .chain
            .iter()
            .position(|h| h.hash == block)
            .and_then(|pos| pos.checked_sub(1))
            .and_then(|pos| self.chain.iter().nth(pos).map(|h| h.hash))
        {
            Ok(hash)
        } else {
            Lookup::parent(&*self.state, block)
        }
    }
}
