//! Streams for the network.
//!
//! Functional handlers for streams.
#![allow(unused)]

use crate::{peer::PeerId, Network};
use quinn::{RecvStream, SendStream};

pub mod ce128;
pub mod ce129;
pub mod ce131;
pub mod ce132;
pub mod ce133;
pub mod ce134;
pub mod ce135;
pub mod ce136;
pub mod ce137;
pub mod ce138;
pub mod ce139;
pub mod ce140;
pub mod ce141;
pub mod ce142;
pub mod ce143;
pub mod ce144;
pub mod ce145;
pub mod up0;

/// Handle an incoming stream.
#[tracing::instrument(skip_all, level = "debug", fields(peer = ?peer.to_string()), name = "stream")]
pub async fn recv<C: score::runtime::Config>(
    peer: PeerId,
    send: SendStream,
    mut recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    let mut buf = [0; 1];
    recv.read_exact(&mut buf).await?;

    match buf[0] {
        0 => up0::recv(peer, send, recv, runtime).await,
        128 => ce128::recv(send, recv, runtime).await,
        129 => ce129::recv(send, recv, runtime).await,
        131 => ce131::recv(send, recv, runtime).await,
        132 => ce132::recv(send, recv, runtime).await,
        133 => ce133::recv(send, recv, runtime).await,
        134 => ce134::recv(send, recv, runtime).await,
        135 => ce135::recv(send, recv, runtime).await,
        136 => ce136::recv(send, recv, runtime).await,
        137 => ce137::recv(send, recv, runtime).await,
        138 => ce138::recv(send, recv, runtime).await,
        139 => ce139::recv(send, recv, runtime).await,
        140 => ce140::recv(send, recv, runtime).await,
        141 => ce141::recv(send, recv, runtime).await,
        142 => ce142::recv(send, recv, runtime).await,
        143 => ce143::recv(send, recv, runtime).await,
        144 => ce144::recv(send, recv, runtime).await,
        145 => ce145::recv(send, recv, runtime).await,
        unknown => anyhow::bail!("unknown stream type: {unknown}"),
    }
}
