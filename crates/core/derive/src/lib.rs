use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Fields, Ident, ItemStruct};

/// Derives a struct to implement the `Json` trait.
///
/// This macro adds a new struct with the `Json` suffix to the original struct.
/// It also modifies the fields of the original struct to be encoded as `String` instead of `[u8; N]`.
#[proc_macro_derive(Json)]
pub fn json_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);

    let mut json = input.clone();

    // 1. Append Json to the struct name
    let name = json.ident.clone();
    let json_name = Ident::new(&format!("{name}Json"), name.span());
    json.ident = json_name.clone();

    // 2. Clean attrs from the original struct except for doc
    json.attrs.retain(|attr| attr.path().is_ident("doc"));

    // 3. Modify [u8; N] fields and fields with the #[json(hex)] attribute to String
    let mut hex_fields = Vec::new();
    let mut other_fields = Vec::new();
    if let Fields::Named(ref mut fields) = json.fields {
        for field in &mut fields.named {
            // Check for the #[json(hex)] attribute
            if field.attrs.iter().any(|attr| {
                attr.path().is_ident("json")
                    && attr
                        .parse_args::<syn::Ident>()
                        .expect("Invalid json attribute")
                        .to_string()
                        == "hex".to_string()
            }) {
                hex_fields.push(field.ident.clone());
                field.ty = syn::parse_quote! { String };
                continue;
            }

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
                    #(#other_fields: value.#other_fields,)*
                    #(#hex_fields: hex::decode(value.#hex_fields.trim_start_matches("0x"))?
                        .try_into()
                        .map_err(|_| anyhow::anyhow!("Invalid hex string"))?,
                    )*
                })
            }
        }

        impl From<#name> for #json_name {
            fn from(value: #name) -> Self {
                #json_name {
                    #(#other_fields: value.#other_fields,)*
                    #(#hex_fields: hex::encode(value.#hex_fields),)*
                }
            }
        }
    }
    .into()
}
