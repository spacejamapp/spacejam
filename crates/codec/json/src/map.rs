use crate::Json;
use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::BTreeMap, fmt::Debug};

impl<K, V> Json<BTreeMap<K, V>> for BTreeMap<K, V>
where
    K: Serialize + DeserializeOwned + Ord + Debug,
    V: Serialize + DeserializeOwned + Debug,
{
    fn to_json(self) -> BTreeMap<K, V> {
        self
    }

    fn from_json(json: BTreeMap<K, V>) -> Result<Self> {
        Ok(json)
    }
}
