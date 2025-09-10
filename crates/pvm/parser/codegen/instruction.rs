use super::Format;
use quote::{quote, ToTokens};
use syn::{parse_quote, Ident, ItemEnum, ItemImpl, Variant};

/// The codegen for the instruction enum.
pub struct InstructionEnum {
    /// The item enum.
    pub item: ItemEnum,
    /// The display implementation.
    pub display_impl: Option<ItemImpl>,
    /// The debug implementation.
    pub debug_impl: Option<ItemImpl>,
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

    /// Generate Display implementation for Instruction enum
    pub fn impl_display(&mut self) {
        let variants = &self.item.variants;
        let mut display_arms = Vec::new();
        let mut debug_arms = Vec::new();

        for variant in variants {
            let variant_name = &variant.ident;

            if variant.fields.is_empty() {
                debug_arms.push(quote! {
                    Self::#variant_name => write!(f, stringify!(#variant_name))
                });
                display_arms.push(quote! {
                    Self::#variant_name => write!(f, stringify!(#variant_name))
                });
            } else {
                debug_arms.push(quote! {
                    Self::#variant_name(format) => write!(f, "{}({})", stringify!(#variant_name), format)
                });
                display_arms.push(quote! {
                    Self::#variant_name(_format) => write!(f, "{}", stringify!(#variant_name))
                });
            }
        }

        // Create the final Display implementation
        self.display_impl = Some(parse_quote! {
            impl std::fmt::Display for Instruction {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#display_arms,)*
                    }
                }
            }
        });

        // Create the final Debug implementation
        self.debug_impl = Some(parse_quote! {
            impl std::fmt::Debug for Instruction {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#debug_arms,)*
                    }
                }
            }
        });
    }
}

impl core::fmt::Display for InstructionEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = self.item.to_token_stream().to_string();

        // Add the Display implementation if it exists
        if let Some(display_impl) = &self.display_impl {
            output.push_str("\n\n");
            output.push_str(&display_impl.to_token_stream().to_string());
        }

        if let Some(debug_impl) = &self.debug_impl {
            output.push_str("\n\n");
            output.push_str(&debug_impl.to_token_stream().to_string());
        }

        write!(f, "{output}")
    }
}

impl Default for InstructionEnum {
    fn default() -> Self {
        let item = parse_quote! {
            /// The PVM instruction enum.
            #[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
            pub enum Instruction {}
        };

        Self {
            item,
            display_impl: None,
            debug_impl: None,
        }
    }
}
