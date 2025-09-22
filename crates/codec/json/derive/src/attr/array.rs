//! Array attribute processing

use crate::field::Fields;
use crate::util;

/// Process array attribute: array(key = "name", value = "name")
pub fn process(call: &syn::ExprCall, field: &mut syn::Field, categories: &mut Fields) -> bool {
    // Check if this is array() function call
    let syn::Expr::Path(ref path) = *call.func else {
        return false;
    };
    if !path.path.is_ident("array") {
        return false;
    }

    // Check if field is a map type
    if !is_map_type(&field.ty) {
        return false;
    }

    // Extract key/value names from arguments
    let Some((key, value)) = extract_key_value_names(&call.args) else {
        return false;
    };

    // Transform field and add to categories
    categories.array_map.push((field.ident.clone(), key, value));
    let struct_name = format!(
        "{}Entry",
        util::to_pascal_case(&field.ident.as_ref().unwrap().to_string())
    );
    let struct_ident = syn::Ident::new(&struct_name, field.ident.as_ref().unwrap().span());
    field.ty = syn::parse_quote!(Vec<#struct_ident>);

    true
}

/// Check if field type is a map (BTreeMap or HashMap)
fn is_map_type(field_ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = field_ty else {
        return false;
    };
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    segment.ident == "BTreeMap" || segment.ident == "HashMap"
}

/// Extract key/value names from array() arguments
fn extract_key_value_names(
    args: &syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>,
) -> Option<(String, String)> {
    let mut key_name = None;
    let mut value_name = None;

    for arg in args {
        let syn::Expr::Assign(assign) = arg else {
            continue;
        };
        let (
            syn::Expr::Path(left),
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(right),
                ..
            }),
        ) = (&*assign.left, &*assign.right)
        else {
            continue;
        };

        if left.path.is_ident("key") {
            key_name = Some(right.value());
        } else if left.path.is_ident("value") {
            value_name = Some(right.value());
        }
    }

    match (key_name, value_name) {
        (Some(key), Some(value)) => Some((key, value)),
        _ => None,
    }
}
