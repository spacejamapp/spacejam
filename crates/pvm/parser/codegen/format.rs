use proc_macro2::Span;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use syn::Ident;

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
    /// The identifier of the format.
    #[serde(skip)]
    pub ident: Option<Ident>,
}

impl Format {
    /// Returns the instruction formats for the PVM.
    pub fn tables() -> Vec<Format> {
        let toml = include_str!("../instruction/v0.4.5.toml");
        let formats: HashMap<String, Format> =
            toml::from_str(toml).expect("Failed to parse formats");

        let formats = formats
            .into_iter()
            .map(|(name, mut format)| {
                if name != "Z" {
                    let ident = Ident::new(&name, Span::call_site());
                    format.ident = Some(ident);
                }

                format
            })
            .collect();

        formats
    }
}

/// An opcode in the PVM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opcode {
    pub name: String,
    pub description: String,
    pub opcode: u8,
}
