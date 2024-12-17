//! The opcode definitions.

use super::format::Opcode;
use quote::quote;
use syn::{parse_quote, Arm, Ident, ItemEnum};

/// The opcode enum.
pub struct OpcodeEnum {
    /// The opcode enum item.
    pub item: ItemEnum,

    /// The try_from_u8 implementation.
    pub try_from_u8_arms: Vec<Arm>,
}

impl OpcodeEnum {
    /// Emits a new opcode.
    pub fn emit(&mut self, opcode: &Opcode, name: &Ident) {
        let index = opcode.opcode;
        let description = &opcode.description;

        // Add the opcode to the enum.
        self.item.variants.push(parse_quote! {
            #[doc = #description]
            #name = #index
        });

        // Add the try_from_u8 implementation.
        self.try_from_u8_arms.push(parse_quote! {
            #index => Ok(Self::#name),
        });
    }
}

impl core::fmt::Display for OpcodeEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let item = self.item.clone();
        let arms = self.try_from_u8_arms.clone();

        let formatted = quote! {
            #item

            impl TryFrom<u8> for Opcode {
                type Error = anyhow::Error;

                fn try_from(value: u8) -> anyhow::Result<Self> {
                    match value {
                        #(#arms)*
                        _ => anyhow::bail!("invalid opcode: {value}"),
                    }
                }
            }
        }
        .to_string();

        write!(f, "{formatted}")
    }
}

impl Default for OpcodeEnum {
    fn default() -> Self {
        let item = parse_quote!(
            /// The opcodes for the PVM.
            #[repr(u8)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum Opcode {}
        );

        Self {
            item,
            try_from_u8_arms: vec![],
        }
    }
}
