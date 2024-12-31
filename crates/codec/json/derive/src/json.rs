use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Fields, Ident, ItemStruct};

/// Derives a struct to implement the `Json` trait.
///
/// This macro adds a new struct with the `Json` suffix to the original struct.
/// It also modifies the fields of the original struct to be encoded as `String`
/// instead of `[u8; N]`.
pub fn derive(input: TokenStream) -> TokenStream {
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

        // Clone the attribute to remove it from the field
        let attr = attr.clone();
        field
            .attrs
            .retain(|attr| !attr.path().is_ident("json") && !attr.path().is_ident("serde"));

        // Parse the attribute
        let Ok(arg) = attr.parse_args::<syn::Ident>() else {
            let ty = attr
                .parse_args::<syn::Path>()
                .expect("invalid json attribute");

            other_fields.push(field.ident.clone());
            field.ty = syn::parse_quote!(#ty);
            continue;
        };

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
                    if inner_ty.starts_with("Option") {
                        field.ty = syn::parse_quote!(Vec<Option<String>>);
                    } else {
                        field.ty = syn::parse_quote!(Vec<String>);
                    }
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
            let syn::PathArguments::AngleBracketed(ref args) = segment.arguments else {
                let nested_ty = Ident::new(
                    &format!("{}Json", field.ty.to_token_stream()),
                    Span::call_site(),
                );

                other_fields.push(field.ident.clone());
                field.ty = syn::parse_quote!(#nested_ty);
                continue;
            };

            let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() else {
                panic!("Invalid type depth");
            };

            nested_array_fields.push(field.ident.clone());
            let nested_ty = Ident::new(
                &format!("{}Json", inner_type.to_token_stream()),
                Span::call_site(),
            );

            if is_option {
                field.ty = syn::parse_quote!(Option<#nested_ty>);
            } else {
                field.ty = syn::parse_quote!(Vec<#nested_ty>);
            }

            continue;
        }
    }

    // 4. Append attributes to the json struct
    json.attrs.extend(vec![syn::parse_quote! {
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
    }]);

    quote! {
        #json

        impl #name {
            /// Parse self from json string
            pub fn from_json(json: impl AsRef<str>) -> anyhow::Result<Self> {
                let jsonf = serde_json::from_str(json.as_ref())?;
                Json::from_json(jsonf)
            }

            /// Parse Vec<Self> from json string
            pub fn load_json(json: impl AsRef<str>) -> anyhow::Result<Vec<Self>> {
                let jsonf: Vec<#json_name> = serde_json::from_str(json.as_ref())?;
                Json::from_json(jsonf)
            }
        }

        impl Json<#json_name> for #name {
            fn to_json(self) -> #json_name {
                #json_name {
                    #(#other_fields: self.#other_fields.to_json(),)*
                    #(#hex_fields: self.#hex_fields.to_json(),)*
                    #(#option_fields: self.#option_fields.to_json(),)*
                    #(#array_fields: self.#array_fields.to_json(),)*
                    #(#nested_array_fields: self.#nested_array_fields.to_json(),)*
                }
            }

            fn from_json(json: #json_name) -> anyhow::Result<Self> {
                Ok(#name {
                    #(#other_fields: Json::from_json(json.#other_fields)?,)*
                    #(#hex_fields: Json::from_json(json.#hex_fields)?,)*
                    #(#option_fields: Json::from_json(json.#option_fields)?,)*
                    #(#array_fields: Json::from_json(json.#array_fields)?,)*
                    #(#nested_array_fields: Json::from_json(json.#nested_array_fields)?,)*
                })
            }
        }

        impl From<#name> for #json_name {
            fn from(value: #name) -> Self {
                value.to_json()
            }
        }

        impl TryFrom<#json_name> for #name {
            type Error = anyhow::Error;

            fn try_from(value: #json_name) -> anyhow::Result<Self> {
                Json::from_json(value)
            }
        }
    }
    .into()
}
