//! Map tests

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Foo {
    map: BTreeMap<u32, u32>,
}

#[test]
fn test_map() {
    let foo = Foo {
        map: BTreeMap::from([(1, 2), (3, 4)]),
    };

    let encoded = serde_jam::encode(&foo).unwrap();
    let decoded = serde_jam::decode::<Foo>(&encoded).unwrap();
    assert_eq!(foo, decoded);
}
