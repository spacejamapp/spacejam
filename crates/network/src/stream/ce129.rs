//! State request stream.
//!
//! TODO: implement this after grandpa.

use std::mem;

use crate::{Context, Network};
use quinn::{RecvStream, SendStream};
use serde::{Deserialize, Serialize};

/// Send a state request.
pub async fn send(
    mut send: SendStream,
    mut recv: RecvStream,
    request: Request,
) -> anyhow::Result<Response> {
    let mut buf = vec![129];
    buf.extend_from_slice(&codec::encode(&request)?);
    send.write_all(&buf).await?;
    send.finish();

    // read the response
    let mut buf = vec![0; request.maximum as usize];
    recv.read_exact(&mut buf).await?;
    let response: Response = codec::decode(&buf[..])?;
    Ok(response)
}

/// Receive a state request.
pub async fn recv<C: Context + Send + Sync + 'static>(
    mut send: SendStream,
    mut recv: RecvStream,
    context: Network<C>,
) -> anyhow::Result<()> {
    let size = mem::size_of::<Request>();
    let mut buf = vec![0; size];
    recv.read_exact(&mut buf).await?;
    let request: Request = codec::decode(&buf[..])?;
    let response = context.context.fetch_state(request)?;
    send.write_all(&codec::encode(&response)?).await?;
    send.finish();
    Ok(())
}

/// A state request.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Deserialize, Serialize)]
pub struct Request {
    /// The hash of block header
    pub hash: [u8; 32],

    /// The start key
    pub start: [u8; 31],

    /// The end key
    pub end: [u8; 31],

    /// The maximum size of the items to request
    pub maximum: u32,
}

/// A state response.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Deserialize, Serialize)]
pub struct Response {
    /// The boundary nodes
    pub path: Vec<[u8; 32]>,

    /// The pairs of key and value
    pub pairs: Vec<Vec<u8>>,
}
