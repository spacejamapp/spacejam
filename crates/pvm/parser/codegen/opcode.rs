//! The opcode definitions.

use super::format::Opcode;
use heck::ToUpperCamelCase;
use proc_macro2::Span;
use quote::{quote, ToTokens};
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
    pub fn emit(&mut self, opcode: &Opcode) {
        let index = opcode.opcode;
        let name = Ident::new(&opcode.name.to_upper_camel_case(), Span::call_site());
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

impl ToString for OpcodeEnum {
    fn to_string(&self) -> String {
        let item = self.item.clone();
        let arms = self.try_from_u8_arms.clone();
        quote! {
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
        .to_token_stream()
        .to_string()
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
