use proc_macro::TokenStream;

mod json;

/// Derives a struct to implement the `Json` trait.
///
/// This macro adds a new struct with the `Json` suffix to the original struct.
/// It also modifies the fields of the original struct to be encoded as `String`
/// instead of `[u8; N]`.
#[proc_macro_derive(Json, attributes(json))]
pub fn json_derive(input: TokenStream) -> TokenStream {
    json::derive(input)
}
