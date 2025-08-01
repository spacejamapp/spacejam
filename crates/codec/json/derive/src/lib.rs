use proc_macro::TokenStream;

mod attr;
mod field;
mod json;
mod util;

/// JSON derive macro - generates clean map-to-array conversions
#[proc_macro_derive(Json, attributes(json))]
pub fn json_derive(input: TokenStream) -> TokenStream {
    json::derive(input)
}
