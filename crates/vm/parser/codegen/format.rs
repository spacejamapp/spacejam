//! The codegen for the formats.

use proc_macro2::Span;
use quote::quote;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use syn::{parse_quote, Field, Ident, ItemImpl, ItemStruct, LitStr, Type};

/// The codegen for the formats.
#[derive(Default)]
pub struct Formats {
    /// The formats.
    pub formats: Vec<ItemStruct>,
    /// The format implementations.
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
            .map(|i| (format!("reg{i}"), parse_quote!(u8), "register"))
            .chain(
                (0..format.immediate)
                    .map(|i| (format!("imm{i}"), parse_quote!(Register), "immediate")),
            )
            .chain(
                (0..format.extended_immediate)
                    .map(|i| (format!("eimm{i}"), parse_quote!(u64), "extended-immediate")),
            )
            .chain((0..format.offset).map(|i| (format!("off{i}"), parse_quote!(i32), "offset")))
            .map(|(name, value, doc): (String, Type, &str)| {
                let i = name.chars().last().expect("Failed to get last char");
                let ident = Ident::new(&name, Span::call_site());
                parse_quote! {
                    #[doc = concat!("The ", #doc, " ", #i, ".") ]
                    pub #ident: #value
                }
            })
            .collect();

        let description = &format.description;
        let item: ItemStruct = parse_quote! {
            #[doc = #description]
            #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
            pub struct #name {
                #(#fields),*
            }
        };

        self.formats.push(item);
        self.impl_display(name, &fields);
    }

    /// Emits the display implementation for a format.
    fn impl_display(&mut self, name: &Ident, fields: &[Field]) {
        // Create field idents for Display implementation
        let field_idents: Vec<Ident> = fields
            .iter()
            .map(|field| field.ident.as_ref().expect("Field has no name").clone())
            .collect();

        // Add Display implementation - much simpler approach
        let impl_display: ItemImpl = if field_idents.is_empty() {
            parse_quote! {
                impl std::fmt::Display for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{} {{}}", stringify!(#name))
                    }
                }
            }
        } else {
            let mut arms = Vec::new();
            for (i, ident) in field_idents.iter().enumerate() {
                let fhex = LitStr::new(
                    if i != 0 {
                        ", {}: 0x{:x}"
                    } else {
                        " {}: 0x{:x}"
                    },
                    Span::call_site(),
                );
                let fdec = LitStr::new(
                    if i != 0 { ", {}: {}" } else { " {}: {}" },
                    Span::call_site(),
                );

                // write the field
                let istr = ident.to_string();
                let display = if istr.starts_with("imm")
                    || istr.starts_with("eimm")
                    || istr.starts_with("off")
                {
                    quote! { write!(f, #fhex, stringify!(#ident), self.#ident)?; }
                } else {
                    quote! { write!(f, #fdec, stringify!(#ident), self.#ident)?; }
                };
                arms.push(display);
            }

            parse_quote! {
                impl std::fmt::Display for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{} {{", stringify!(#name))?;
                        #(#arms)*
                        write!(f, " }}")
                    }
                }
            }
        };

        self.impls.push(impl_display);
    }
}

impl core::fmt::Display for Formats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formats = self.formats.clone();
        let impls = self.impls.clone();
        let formatted = quote! {
            #(#formats)*

            #(#impls)*
        }
        .to_string();

        write!(f, "{formatted}")
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
    /// The number of extended immediate arguments in the format.
    #[serde(rename = "extended-immediate")]
    pub extended_immediate: u8,
    /// The opcodes in the format.
    pub opcodes: Vec<Opcode>,
    /// The identifier of the format.
    #[serde(skip)]
    pub ident: Option<Ident>,
}

impl Format {
    /// Returns the instruction formats for the PVM.
    pub fn tables() -> Vec<Format> {
        let toml = include_str!("../instruction/v0.5.4.toml");
        let formats: HashMap<String, Format> =
            toml::from_str(toml).expect("Failed to parse formats");

        formats
            .into_iter()
            .map(|(name, mut format)| {
                if name != "Z" {
                    let ident = Ident::new(&name, Span::call_site());
                    format.ident = Some(ident);
                }

                format
            })
            .collect()
    }
}

/// An opcode in the PVM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opcode {
    /// The name of the opcode.
    pub name: String,
    /// The description of the opcode.
    pub description: String,
    /// The opcode.
    pub opcode: u8,
}
