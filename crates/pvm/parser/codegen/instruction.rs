use quote::ToTokens;
use syn::{parse_quote, Ident, ItemEnum, Variant};

use super::Format;

/// The codegen for the instruction enum.
pub struct InstructionEnum {
    /// The item enum.
    pub item: ItemEnum,
}

impl InstructionEnum {
    /// Emits a new instruction.
    pub fn emit(&mut self, format: &Format, opcode: &Ident) {
        // Add the opcode to the enum.
        let mut variant: Variant = if let Some(format) = &format.ident {
            parse_quote!(#opcode(#format))
        } else {
            parse_quote!(#opcode)
        };

        let desc = format.description.clone();
        variant.attrs.append(&mut vec![
            parse_quote!(#[doc = concat!("The ", stringify!(#opcode), " instruction.")]),
            parse_quote!(#[doc = ""]),
            parse_quote!(#[doc = concat!("Format: ", #desc, ".")]),
        ]);
        self.item.variants.push(variant);
    }
}

impl core::fmt::Display for InstructionEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatted = self.item.to_token_stream().to_string();
        write!(f, "{formatted}")
    }
}

impl Default for InstructionEnum {
    fn default() -> Self {
        let item = parse_quote! {
            /// The PVM instruction enum.
            pub enum Instruction {}
        };

        Self { item }
    }
}
