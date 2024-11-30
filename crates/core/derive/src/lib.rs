use proc_macro::TokenStream;
use quote::ToTokens;
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
    let name = &json.ident;
    json.ident = Ident::new(&format!("{name}Json"), name.span());

    // 2. Clean attrs from the original struct except for doc
    json.attrs.retain(|attr| attr.path().is_ident("doc"));

    // 3. Modify [u8; N] fields to String
    //
    // TODO: support this encoding with attributes on fields.
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
                field.ty = syn::parse_quote! { String }; // Change to String
                continue; // Skip further checks for this field
            }

            let syn::Type::Array(ref array_type) = field.ty else {
                continue; // Skip if not an array type
            };
            let syn::Type::Path(ref path_type) = *array_type.elem else {
                continue; // Skip if not a path type
            };
            if path_type.path.is_ident("u8") {
                field.ty = syn::parse_quote! { String };
            }
        }
    }

    json.to_token_stream().into()
}
