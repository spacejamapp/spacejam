//! The visitor for the PVM parser.

use super::format::{Format, Opcode};
use heck::ToSnakeCase;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{parse_quote, Arm, Ident, ItemTrait};

/// The visitor trait.
pub struct VisitorTrait {
    /// The item trait.
    pub item: ItemTrait,

    /// The implementation arms.
    pub impl_visit_arms: Vec<Arm>,

    /// Dispatch information.
    pub dispatch: Vec<(Format, Ident)>,
}

impl VisitorTrait {
    /// Emits a new visitor trait.
    pub fn emit(&mut self, format: &Format, opcode: &Opcode, opcodei: &Ident) {
        let fun = Ident::new(
            &format!("visit_{}", opcode.name.to_snake_case()),
            Span::call_site(),
        );
        let name = opcode.name.clone();
        self.dispatch.push((format.clone(), fun.clone()));

        // Generate the visit functions
        if let Some(format) = &format.ident {
            self.item.items.push(parse_quote! {
                #[doc = concat!("Visits an ", #name, " instruction.")]
                fn #fun(&mut self, _format: #format, _range: &core::ops::Range<usize>) -> Result<Self::Output, Self::Error> {
                    self.visit_default()
                }
            });

            self.impl_visit_arms
                .push(parse_quote!(Instruction::#opcodei(fmt) => self.#fun(fmt, range),));
        } else {
            self.item.items.push(parse_quote! {
                #[doc = concat!("Visits an ", #name, " instruction.")]
                fn #fun(&mut self, _range: &core::ops::Range<usize>) -> Result<Self::Output, Self::Error> {
                    self.visit_default()
                }
            });
            self.impl_visit_arms
                .push(parse_quote!(Instruction::#opcodei => self.#fun(range),));
        }
    }
}

impl core::fmt::Display for VisitorTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut item = self.item.clone();
        let impl_visit_arms = self.impl_visit_arms.clone();
        item.items.push(parse_quote! {
            /// Visits an instruction.
            fn visit(&mut self, instruction: Instruction, range: &core::ops::Range<usize>) -> Result<Self::Output, Self::Error> {
                match instruction {
                    #(#impl_visit_arms)*
                }
            }
        });

        // Generate dispatch table entries - zero-match using calculated format size
        let dispatch_table: Vec<_> = self
            .dispatch
            .iter()
            .map(|(format, visit_fn)| {
                if let Some(format_type) = &format.ident {
                    quote! {
                        |visitor, instruction, range| {
                            let format_data = unsafe {
                                let layout_ptr = &instruction as *const Instruction as *const (u8, #format_type);
                                (*layout_ptr).1
                            };
                            visitor.#visit_fn(format_data, range)
                        }
                    }
                } else {
                    quote! {
                        |visitor, _instruction, range| visitor.#visit_fn(range)
                    }
                }
            })
            .collect();

        let table_len = dispatch_table.len();
        item.items.push(parse_quote! {
            /// Dispatch table for visitor pattern
            #[allow(clippy::type_complexity)]
            const DISPATCH_TABLE: [fn(&mut Self, Instruction, &core::ops::Range<usize>) -> Result<Self::Output, Self::Error>; #table_len] = [
                #(#dispatch_table,)*
            ];
        });

        // Generate the dispatch function
        item.items.push(parse_quote! {
            #[inline(always)]
            fn dispatch(&mut self, instruction: Instruction, range: &core::ops::Range<usize>) -> Result<Self::Output, Self::Error> {
                let idx = unsafe { *((&instruction) as *const Instruction as *const u8) as usize };
                (Self::DISPATCH_TABLE[idx])(self, instruction, range)
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
            pub trait Visitor {
                /// The error type.
                type Error;

                /// The output type.
                type Output;

                /// The default handler for unknown instructions.
                fn visit_default(&mut self) -> Result<Self::Output, Self::Error> {
                    unimplemented!("implement `default` for adapting the unknown instruction")
                }
            }
        };

        Self {
            item,
            impl_visit_arms: vec![],
            dispatch: vec![],
        }
    }
}
