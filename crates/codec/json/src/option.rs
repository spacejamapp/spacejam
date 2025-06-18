use crate::Json;
use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

impl<M: Serialize + DeserializeOwned, N> Json<Option<M>> for Option<N>
where
    N: Json<M>,
{
    fn to_json(self) -> Option<M> {
        self.map(|v| v.to_json())
    }

    fn from_json(json: Option<M>) -> Result<Self> {
        json.map(|v| N::from_json(v)).transpose()
    }
}
