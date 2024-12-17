//! The visitor for the PVM parser.

use super::format::Opcode;
use heck::ToSnakeCase;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{parse_quote, Ident, ItemTrait};

/// The visitor trait.
pub struct VisitorTrait {
    /// The item trait.
    pub item: ItemTrait,
}

impl VisitorTrait {
    /// Emits a new visitor trait.
    pub fn emit(&mut self, format: &Option<Ident>, opcode: &Opcode) {
        let fun = Ident::new(
            &format!("visit_{}", opcode.name.to_snake_case()),
            Span::call_site(),
        );
        let name = opcode.name.clone();

        if let Some(format) = format {
            self.item.items.push(parse_quote! {
                #[doc = concat!("Visits an ", #name, " instruction.")]
                fn #fun(&mut self, format: &#format) -> anyhow::Result<()>;
            });
        } else {
            self.item.items.push(parse_quote! {
                #[doc = concat!("Visits an ", #name, " instruction.")]
                fn #fun(&mut self) -> anyhow::Result<()>;
            });
        }
    }
}

impl ToString for VisitorTrait {
    fn to_string(&self) -> String {
        self.item.to_token_stream().to_string()
    }
}

impl Default for VisitorTrait {
    fn default() -> Self {
        let item = parse_quote! {
            /// The PVM instruction visitor.
            pub trait Visitor {
                /* /// Visits an instruction.
                fn visit(&self, instruction: &Instruction) -> anyhow::Result<()>; */
            }
        };

        Self { item }
    }
}
