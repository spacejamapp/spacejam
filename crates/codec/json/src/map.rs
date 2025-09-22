use crate::Json;
use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    hash::Hash,
};

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

impl<K, V> Json<Vec<(K, V)>> for BTreeMap<K, V>
where
    K: Serialize + DeserializeOwned + Ord + Debug,
    V: Serialize + DeserializeOwned + Debug,
{
    fn to_json(self) -> Vec<(K, V)> {
        self.into_iter().collect()
    }

    fn from_json(json: Vec<(K, V)>) -> Result<Self> {
        Ok(json.into_iter().collect())
    }
}

impl<K, V> Json<HashMap<K, V>> for HashMap<K, V>
where
    K: Serialize + DeserializeOwned + Eq + Hash + Debug,
    V: Serialize + DeserializeOwned + Debug,
{
    fn to_json(self) -> HashMap<K, V> {
        self
    }

    fn from_json(json: HashMap<K, V>) -> Result<Self> {
        Ok(json)
    }
}

impl<K, V> Json<Vec<(K, V)>> for HashMap<K, V>
where
    K: Serialize + DeserializeOwned + Eq + Hash + Debug,
    V: Serialize + DeserializeOwned + Debug,
{
    fn to_json(self) -> Vec<(K, V)> {
        self.into_iter().collect()
    }

    fn from_json(json: Vec<(K, V)>) -> Result<Self> {
        Ok(json.into_iter().collect())
    }
}
