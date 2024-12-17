use anyhow::Result;
use heck::ToUpperCamelCase;
use proc_macro2::Span;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use syn::{parse_quote, Expr, Ident};

/// A instruction format in the PVM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Format {
    /// The description of the format.
    pub description: String,
    /// The number of registers in the format.
    pub register: u8,
    /// The number of immediate arguments in the format.
    pub immediate: u8,
    /// The number of offset arguments in the format.
    pub offset: u8,
    /// The opcodes in the format.
    pub opcodes: Vec<Opcode>,
}

impl Format {
    /// Returns the instruction formats for the PVM.
    pub fn tables() -> Result<HashMap<Ident, Format>> {
        let toml = include_str!("../instruction/v0.4.5.toml");
        let formats: HashMap<String, Format> = toml::from_str(toml)?;

        // Convert the format names to identifiers.
        let mut map = HashMap::new();
        for (name, format) in formats {
            let ident = Ident::new(&name, Span::call_site());
            map.insert(ident, format);
        }

        Ok(map)
    }
}

/// An opcode in the PVM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opcode {
    pub name: String,
    pub description: String,
    pub opcode: u8,
}
