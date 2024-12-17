use proc_macro2::Span;
use quote::quote;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use syn::{parse_quote, Field, Ident, ItemImpl, ItemStruct};

/// The codegen for the formats.
pub struct Formats {
    /// The formats.
    pub formats: Vec<ItemStruct>,

    /// The impls.
    pub impls: Vec<ItemImpl>,
}

impl Formats {
    /// Emits a new format.
    pub fn emit(&mut self, name: &Option<Ident>, format: &Format) {
        let Some(name) = name else {
            return;
        };

        // introduce the format struct
        let fields: Vec<Field> = (0..format.register)
            .map(|i| (format!("reg{}", i), "register"))
            .chain((0..format.immediate).map(|i| (format!("imm{}", i), "immediate")))
            .chain((0..format.offset).map(|i| (format!("off{}", i), "offset")))
            .map(|(name, doc)| {
                let i = name.chars().last().expect("Failed to get last char");
                let ident = Ident::new(&name, Span::call_site());
                parse_quote! {
                    #[doc = concat!("The ", #doc, " ", #i, ".") ]
                    pub #ident: u8
                }
            })
            .collect();

        let description = &format.description;
        let item: ItemStruct = parse_quote! {
            #[doc = #description]
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub struct #name {
                #(#fields),*
            }
        };

        self.formats.push(item);

        // implement the format struct
    }
}

impl ToString for Formats {
    fn to_string(&self) -> String {
        let formats = self.formats.clone();
        let impls = self.impls.clone();

        quote! {
            #(#formats)*

            #(#impls)*
        }
        .to_string()
    }
}

impl Default for Formats {
    fn default() -> Self {
        Self {
            formats: Vec::new(),
            impls: Vec::new(),
        }
    }
}

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
