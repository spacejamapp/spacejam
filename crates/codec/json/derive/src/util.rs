//! Utility functions for the JSON derive macro

/// Convert a field name to PascalCase for struct names
pub fn to_pascal_case(s: &str) -> String {
    s.chars()
        .next()
        .map(|c| c.to_uppercase().collect::<String>())
        .unwrap_or_default()
        + &s[1..]
}
