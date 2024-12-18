//! The visitor for the PVM parser.

use super::format::Opcode;
use heck::ToSnakeCase;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{parse_quote, Arm, Ident, ItemTrait};

/// The visitor trait.
pub struct VisitorTrait {
    /// The item trait.
    pub item: ItemTrait,

    /// The implementation arms.
    pub impl_vist_arms: Vec<Arm>,
}

impl VisitorTrait {
    /// Emits a new visitor trait.
    pub fn emit(&mut self, format: &Option<Ident>, opcode: &Opcode, opcodei: &Ident) {
        let fun = Ident::new(
            &format!("visit_{}", opcode.name.to_snake_case()),
            Span::call_site(),
        );
        let name = opcode.name.clone();

        if let Some(format) = format {
            self.item.items.push(parse_quote! {
                #[doc = concat!("Visits an ", #name, " instruction.")]
                fn #fun(&mut self, _format: #format) -> anyhow::Result<()> {
                    unimplemented!(concat!("visit_", #name, " not implemented"))
                }
            });

            self.impl_vist_arms
                .push(parse_quote!(Instruction::#opcodei(fmt) => self.#fun(fmt),));
        } else {
            self.item.items.push(parse_quote! {
                #[doc = concat!("Visits an ", #name, " instruction.")]
                fn #fun(&mut self) -> anyhow::Result<()> {
                    unimplemented!(concat!("visit_", #name, " not implemented"))
                }
            });
            self.impl_vist_arms
                .push(parse_quote!(Instruction::#opcodei => self.#fun(),));
        }
    }
}

impl core::fmt::Display for VisitorTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut item = self.item.clone();
        let impl_vist_arms = self.impl_vist_arms.clone();

        item.items.push(parse_quote! {
            /// Visits an instruction.
            fn visit(&mut self, instruction: Instruction) -> anyhow::Result<()> {
                match instruction {
                    #(#impl_vist_arms)*
                }
            }
        });

        let formatted = item.to_token_stream().to_string();
        write!(f, "{formatted}")
    }
}

impl Default for VisitorTrait {
    fn default() -> Self {
        let item = parse_quote! {
            /// The PVM instruction visitor.
            pub trait Visitor {}
        };

        Self {
            item,
            impl_vist_arms: vec![],
        }
    }
}
