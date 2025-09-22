use crate::Json;
use anyhow::Result;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// A JSON representation of a `Result`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultJson<M, N> {
    /// The OK value.
    pub ok: Option<M>,
    /// The error value.
    pub err: Option<N>,
}

impl<M: Serialize + DeserializeOwned, N: Serialize + DeserializeOwned, P, Q> Json<ResultJson<M, N>>
    for core::result::Result<P, Q>
where
    P: Json<M>,
    Q: Json<N>,
{
    fn to_json(self) -> ResultJson<M, N> {
        if self.is_ok() {
            ResultJson {
                ok: self.ok().to_json(),
                err: None,
            }
        } else {
            ResultJson {
                ok: None,
                err: self.err().to_json(),
            }
        }
    }

    fn from_json(json: ResultJson<M, N>) -> Result<Self> {
        if let Some(ok) = json.ok {
            Ok(Ok(P::from_json(ok)?))
        } else if let Some(err) = json.err {
            Ok(Err(Q::from_json(err)?))
        } else {
            Err(anyhow::anyhow!("Invalid result JSON"))
        }
    }
}
