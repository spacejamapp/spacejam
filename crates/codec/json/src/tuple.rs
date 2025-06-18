//! Tuple implementations

use crate::Json;
use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

impl<M: Serialize + DeserializeOwned + Debug, N: Serialize + DeserializeOwned + Debug, P, Q>
    Json<(M, N)> for (P, Q)
where
    P: Json<M>,
    Q: Json<N>,
{
    fn to_json(self) -> (M, N) {
        (self.0.to_json(), self.1.to_json())
    }

    fn from_json(json: (M, N)) -> Result<Self> {
        Ok((P::from_json(json.0)?, Q::from_json(json.1)?))
    }
}
