use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Fields, Ident, ItemStruct};

/// Derives a struct to implement the `Json` trait.
///
/// This macro adds a new struct with the `Json` suffix to the original struct.
/// It also modifies the fields of the original struct to be encoded as `String`
/// instead of `[u8; N]`.
#[proc_macro_derive(Json, attributes(json))]
pub fn json_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);

    let mut json = input.clone();

    // 1. Append Json to the struct name
    let name = json.ident.clone();
    let json_name = Ident::new(&format!("{name}Json"), name.span());
    json.ident = json_name.clone();

    // 2. Clean attrs from the original struct except for doc
    json.attrs.retain(|attr| attr.path().is_ident("doc"));

    // 3. Modify fields based on attributes
    let mut hex_fields = Vec::new();
    let mut option_fields = Vec::new();
    let mut array_fields = Vec::new();
    let mut nested_array_fields = Vec::new();
    let mut other_fields = Vec::new();
    let Fields::Named(ref mut fields) = json.fields else {
        panic!("Invalid fields");
    };

    for field in &mut fields.named {
        let Some(attr) = field.attrs.iter().find(|attr| attr.path().is_ident("json")) else {
            // Handle [u8; N] fields
            if let syn::Type::Array(ref array_type) = field.ty {
                if let syn::Type::Path(ref path_type) = *array_type.elem {
                    if path_type.path.is_ident("u8") {
                        hex_fields.push(field.ident.clone());
                        field.ty = syn::parse_quote! { String };
                        continue;
                    }
                }
            }

            other_fields.push(field.ident.clone());
            continue;
        };

        let arg = attr
            .parse_args::<syn::Ident>()
            .expect("invalid json attribute");

        field.attrs.retain(|attr| !attr.path().is_ident("json"));

        let syn::Type::Path(ref path) = &field.ty else {
            continue;
        };

        // If it's a Vec<T>, we need to handle it
        let Some(segment) = path.path.segments.last() else {
            continue;
        };

        let is_option = segment.ident == "Option";

        // Check for the #[json(hex)] attribute
        if arg == *"hex" {
            if segment.ident == "Vec" {
                let syn::PathArguments::AngleBracketed(ref args) = segment.arguments else {
                    panic!("Invalid json attribute");
                };

                let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() else {
                    panic!("Invalid type depth");
                };

                let inner_ty = inner_type.to_token_stream().to_string();
                if inner_ty != *"u8" {
                    array_fields.push(field.ident.clone());
                    field.ty = syn::parse_quote!(Vec<String>);
                    continue;
                }
            }

            if is_option {
                option_fields.push(field.ident.clone());
                field.ty = syn::parse_quote!(Option<String>);
                continue;
            } else {
                hex_fields.push(field.ident.clone());
                field.ty = syn::parse_quote! { String };
                continue;
            }
        }

        // Check for the #[json(nested)] attribute
        if arg == *"nested" {
            if segment.ident == "Vec" {
                let syn::PathArguments::AngleBracketed(ref args) = segment.arguments else {
                    panic!("Invalid json attribute");
                };

                let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() else {
                    panic!("Invalid type depth");
                };

                nested_array_fields.push(field.ident.clone());
                let nested_ty = Ident::new(
                    &format!("{}Json", inner_type.to_token_stream()),
                    Span::call_site(),
                );
                field.ty = syn::parse_quote!(Vec<#nested_ty>);
                continue;
            }

            other_fields.push(field.ident.clone());
            let nested_ty = Ident::new(
                &format!("{}Json", field.ty.to_token_stream()),
                Span::call_site(),
            );

            if is_option {
                field.ty = syn::parse_quote!(Option<#nested_ty>);
            } else {
                field.ty = syn::parse_quote!(#nested_ty);
            }
        }
    }

    // 4. Append attributes to the json struct
    json.attrs.extend(vec![syn::parse_quote! {
        #[derive(serde::Serialize, serde::Deserialize)]
    }]);

    quote! {
        #json

        impl TryFrom<#json_name> for #name {
            type Error = anyhow::Error;

            fn try_from(value: #json_name) -> anyhow::Result<Self> {
                Ok(#name {
                    #(#other_fields: value.#other_fields.try_into()?,)*
                    #(#hex_fields: hex::decode(value.#hex_fields.trim_start_matches("0x"))?
                        .try_into()
                        .map_err(|_| anyhow::anyhow!("Invalid hex string"))?,
                    )*
                    #(#option_fields: value.#option_fields.map(|v| hex::decode(v.trim_start_matches("0x")).unwrap_or_default())
                        .try_into()
                        .ok()
                        .flatten(),)*
                    #(#array_fields: value.#array_fields.into_iter().map(|v| {
                        hex::decode(v.trim_start_matches("0x"))
                            .map(|v| v.try_into().map_err(|_| anyhow::anyhow!("Invalid hex string")))
                            .map_err(|_| anyhow::anyhow!("Invalid hex string"))
                        })
                    .collect::<anyhow::Result<anyhow::Result<Vec<_>>>>()??,
                    )*
                    #(#nested_array_fields: value.#nested_array_fields.into_iter().map(|v| {
                        v.try_into().map_err(|_| anyhow::anyhow!("Invalid nested array"))
                    }).collect::<anyhow::Result<Vec<_>>>()?,)*
                })
            }
        }

        impl From<#name> for #json_name {
            fn from(value: #name) -> Self {
                #json_name {
                    #(#other_fields: value.#other_fields.into(),)*
                    #(#hex_fields: hex::encode(value.#hex_fields),)*
                    #(#option_fields: value.#option_fields.map(|v| hex::encode(v)).unwrap_or_default().into(),)*
                    #(#array_fields: value.#array_fields.into_iter().map(hex::encode).collect(),)*
                    #(#nested_array_fields: value.#nested_array_fields.into_iter().map(|v| v.into()).collect(),)*
                }
            }
        }
    }
    .into()
}
