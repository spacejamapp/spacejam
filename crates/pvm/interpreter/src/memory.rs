//! Memory management for the interpreter

use std::collections::BTreeMap;

/// The memory of the interpreter.
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct Memory {
    /// The pages of the memory.
    pub pages: BTreeMap<u32, Page>,
    /// The slots of the memory.
    pub slots: BTreeMap<u32, Vec<u8>>,
}

/// A memory page.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Page {
    /// The length of the page.
    pub length: u32,
    /// The access type of the page.
    pub access: Access,
}

/// The access type of a memory page.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Access {
    /// The page is mutable.
    Mutable,
    /// The page is immutable.
    Immutable,
    /// The page is inaccessible.
    Inaccessible,
}
