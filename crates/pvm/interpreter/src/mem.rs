//! Memory management for the interpreter

use std::collections::BTreeMap;

/// The memory of the interpreter.
pub type Memory = BTreeMap<u32, Page>;

/// A memory page.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Page {
    /// The length of the page.
    pub length: u32,
    /// The access type of the page.
    pub access: Access,
    /// The contents of the page.
    pub contents: Vec<u8>,
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
