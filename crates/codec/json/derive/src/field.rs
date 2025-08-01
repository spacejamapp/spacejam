//! Field processing for JSON derive macro

use crate::{attr, util};
use quote::{quote, ToTokens};
use syn::Ident;

/// Categorized field collections for JSON transformation
#[derive(Default)]
pub struct Fields {
    // Fields that are encoded as hex strings
    pub hex: Vec<Option<Ident>>,
    // Fields that are encoded as options
    pub option: Vec<Option<Ident>>,
    // Fields that are encoded as arrays
    pub array: Vec<Option<Ident>>,
    // Fields that are encoded as nested arrays
    pub nested_array: Vec<Option<Ident>>,
    // Fields that are encoded as compact strings
    pub compact: Vec<Option<Ident>>,
    // Fields that are encoded as structs
    pub map_to_struct: Vec<(Option<Ident>, syn::Path)>,
    // Fields that are encoded as arrays of structs
    pub array_map: Vec<(Option<Ident>, String, String)>,
    // Fields that are encoded as other types
    pub other: Vec<Option<Ident>>,
}

impl Fields {
    /// Generate map entries for the array map
    pub fn map_entry(&self) -> MapEntry {
        MapEntry::from(&self.array_map)
    }
}

impl From<&mut syn::FieldsNamed> for Fields {
    /// Process all fields in the struct
    fn from(fields: &mut syn::FieldsNamed) -> Self {
        let mut categories = Fields::default();

        for field in &mut fields.named {
            // Check for json attribute
            if let Some(attr) = field.attrs.iter().find(|attr| attr.path().is_ident("json")) {
                let attr = attr.clone();

                // Remove json and related serde attributes
                field.attrs.retain(|attr| {
                    if attr.path().is_ident("json") {
                        return false;
                    }
                    if attr.path().is_ident("serde") {
                        if let Ok(expr) = attr.parse_args::<syn::Expr>() {
                            if expr.to_token_stream().to_string().contains("with") {
                                return false;
                            }
                        }
                    }
                    true
                });

                if attr::json::process(&attr, field, &mut categories) {
                    continue;
                }
            }

            // Handle [u8; N] fields automatically
            if let syn::Type::Array(ref array_type) = field.ty {
                if let syn::Type::Path(ref path_type) = *array_type.elem {
                    if path_type.path.is_ident("u8") {
                        categories.hex.push(field.ident.clone());
                        field.ty = syn::parse_quote!(String);
                        continue;
                    }
                }
            }

            categories.other.push(field.ident.clone());
        }

        categories
    }
}

/// Consolidated array map generation result
pub struct MapEntry {
    // Inline structs for the array map
    pub inline_structs: Vec<proc_macro2::TokenStream>,
    // To JSON conversions for the array map
    pub to_json: Vec<proc_macro2::TokenStream>,
    // From JSON conversions for the array map
    pub from_json: Vec<proc_macro2::TokenStream>,
}

impl From<&Vec<(Option<Ident>, String, String)>> for MapEntry {
    fn from(array: &Vec<(Option<Ident>, String, String)>) -> Self {
        let mut inline_structs = Vec::new();
        let mut to_json = Vec::new();
        let mut from_json = Vec::new();

        for (field_name, key_name, value_name) in array {
            let struct_name = syn::Ident::new(
                &format!(
                    "{}Entry",
                    util::to_pascal_case(&field_name.as_ref().unwrap().to_string())
                ),
                field_name.as_ref().unwrap().span(),
            );
            let key_field = syn::Ident::new(key_name, field_name.as_ref().unwrap().span());
            let value_field = syn::Ident::new(value_name, field_name.as_ref().unwrap().span());

            // Generate inline struct
            inline_structs.push(quote! {
                #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
                struct #struct_name {
                    #key_field: String,
                    #value_field: String,
                }
            });

            // Generate to_json conversion
            to_json.push(quote! {
                #field_name: self.#field_name.into_iter().map(|(k, v)| {
                    use spacejson::Json;
                    #struct_name {
                        #key_field: k.to_json(),
                        #value_field: v.to_json(),
                    }
                }).collect()
            });

            // Generate from_json conversion
            from_json.push(quote! {
                #field_name: json.#field_name.into_iter().map(|entry| {
                    use spacejson::Json;
                    let key = Json::from_json(entry.#key_field)?;
                    let value = Json::from_json(entry.#value_field)?;
                    Ok((key, value))
                }).collect::<anyhow::Result<std::collections::BTreeMap<_, _>>>()?
            });
        }

        MapEntry {
            inline_structs,
            to_json,
            from_json,
        }
    }
}
