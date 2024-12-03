//! Safrole vector tests
#![cfg(test)]

#[allow(unused_macros)]
macro_rules! impl_safrole_tests {
    ($name:ident) => {
        let json = include_str!(concat!(
            "../jamtestvectors/safrole/tiny/",
            stringify!($name),
            ".json"
        ));
        let data = include_bytes!(concat!(
            "../jamtestvectors/safrole/tiny/",
            stringify!($name),
            ".bin"
        ));
        (json, data)
    };
}
