//! JSON attribute processing

use crate::{attr::array, field::Fields};
use proc_macro2::Span;
use quote::ToTokens;
use syn::Ident;

/// Process regular json attributes (hex, nested, compact)
pub fn process(attr: &syn::Attribute, field: &mut syn::Field, categories: &mut Fields) -> bool {
    let Ok(arg) = attr.parse_args::<syn::Ident>() else {
        // Try function call syntax first
        if let Ok(call) = attr.parse_args::<syn::ExprCall>() {
            return array::process(&call, field, categories);
        }

        // Handle Vec<StructType> syntax
        let ty = attr
            .parse_args::<syn::Path>()
            .expect("invalid json attribute");
        if let syn::Type::Path(path) = &field.ty
            && let Some(segment) = path.path.segments.last()
            && (segment.ident == "BTreeMap" || segment.ident == "HashMap")
        {
            categories
                .map_to_struct
                .push((field.ident.clone(), ty.clone()));
            field.ty = syn::parse_quote!(#ty);
            return true;
        }

        categories.other.push(field.ident.clone());
        field.ty = syn::parse_quote!(#ty);
        return true;
    };

    let syn::Type::Path(path) = &field.ty else {
        return false;
    };
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    let is_option = segment.ident == "Option";

    match arg.to_string().as_str() {
        "compact" if segment.ident == "Compact" => {
            let syn::PathArguments::AngleBracketed(ref args) = segment.arguments else {
                panic!("Invalid compact type");
            };
            let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() else {
                panic!("Invalid compact type");
            };
            categories.compact.push(field.ident.clone());
            field.ty = inner_type.clone();
        }
        "hex" => {
            if segment.ident == "Vec" {
                let syn::PathArguments::AngleBracketed(ref args) = segment.arguments else {
                    panic!("Invalid json attribute");
                };
                let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() else {
                    panic!("Invalid type depth");
                };
                let inner_ty = inner_type.to_token_stream().to_string();
                if inner_ty != *"u8" {
                    categories.array.push(field.ident.clone());
                    field.ty = if inner_ty.starts_with("Option") {
                        syn::parse_quote!(Vec<Option<String>>)
                    } else {
                        syn::parse_quote!(Vec<String>)
                    };
                } else {
                    categories.hex.push(field.ident.clone());
                    field.ty = syn::parse_quote!(String);
                }
            } else if is_option {
                categories.option.push(field.ident.clone());
                field.ty = syn::parse_quote!(Option<String>);
            } else {
                categories.hex.push(field.ident.clone());
                field.ty = syn::parse_quote!(String);
            }
        }
        "nested" => {
            let syn::PathArguments::AngleBracketed(ref args) = segment.arguments else {
                let nested_ty = Ident::new(
                    &format!("{}Json", field.ty.to_token_stream()),
                    Span::call_site(),
                );
                categories.other.push(field.ident.clone());
                field.ty = syn::parse_quote!(#nested_ty);
                return true;
            };
            let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() else {
                panic!("Invalid type depth");
            };
            categories.nested_array.push(field.ident.clone());
            let nested_ty = Ident::new(
                &format!("{}Json", inner_type.to_token_stream()),
                Span::call_site(),
            );
            field.ty = if is_option {
                syn::parse_quote!(Option<#nested_ty>)
            } else {
                syn::parse_quote!(Vec<#nested_ty>)
            };
        }
        _ => return false,
    }
    true
}
