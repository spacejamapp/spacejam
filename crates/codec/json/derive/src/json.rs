use crate::field::{Fields, MapEntry};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Fields as SynFields, Ident, ItemStruct, parse_macro_input};

/// JSON derive macro - generates clean map-to-array conversions
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let mut json = input.clone();

    // Setup JSON struct
    let name = json.ident.clone();
    let json_name = Ident::new(&format!("{name}Json"), name.span());
    json.ident = json_name.clone();
    json.attrs.retain(|attr| attr.path().is_ident("doc"));
    json.attrs
        .push(syn::parse_quote!(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]));

    // Process fields
    let SynFields::Named(ref mut fields) = json.fields else {
        panic!("Invalid fields");
    };
    let fields = Fields::from(fields);
    let map_entry = fields.map_entry();
    let map_field_names: Vec<_> = fields.map_to_struct.iter().map(|(name, _)| name).collect();

    // Extract field vectors and map entry components
    let Fields {
        hex,
        option,
        array,
        nested_array,
        compact,
        other,
        ..
    } = fields;

    let MapEntry {
        inline_structs,
        to_json,
        from_json,
    } = map_entry;

    quote! {
        #(#inline_structs)*
        #json

        impl #name {
            /// Deserialize the JSON string into the struct
            pub fn from_json(json: impl AsRef<str>) -> anyhow::Result<Self> {
                Json::from_json(serde_json::from_str(json.as_ref())?)
            }

            /// Deserialize the JSON string into a vector of structs
            pub fn load_json(json: impl AsRef<str>) -> anyhow::Result<Vec<Self>> {
                Json::from_json(serde_json::from_str::<Vec<#json_name>>(json.as_ref())?)
            }
        }

        impl Json<#json_name> for #name {
            fn to_json(self) -> #json_name {
                #json_name {
                    #(#other: self.#other.to_json(),)*
                    #(#hex: self.#hex.to_json(),)*
                    #(#option: self.#option.to_json(),)*
                    #(#array: self.#array.to_json(),)*
                    #(#nested_array: self.#nested_array.to_json(),)*
                    #(#compact: (*self.#compact),)*
                    #(#map_field_names: self.#map_field_names.to_json(),)*
                    #(#to_json,)*
                }
            }
            fn from_json(json: #json_name) -> anyhow::Result<Self> {
                Ok(#name {
                    #(#other: Json::from_json(json.#other)?,)*
                    #(#hex: Json::from_json(json.#hex)?,)*
                    #(#option: Json::from_json(json.#option)?,)*
                    #(#array: Json::from_json(json.#array)?,)*
                    #(#nested_array: Json::from_json(json.#nested_array)?,)*
                    #(#compact: codec::Compact::from(json.#compact),)*
                    #(#map_field_names: Json::from_json(json.#map_field_names)?,)*
                    #(#from_json,)*
                })
            }
        }

        impl From<#name> for #json_name { fn from(value: #name) -> Self { value.to_json() } }
        impl TryFrom<#json_name> for #name {
            type Error = anyhow::Error;
            fn try_from(value: #json_name) -> anyhow::Result<Self> { Json::from_json(value) }
        }
    }
    .into()
}
