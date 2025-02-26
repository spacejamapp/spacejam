//! Streams for the network.
//!
//! Functional handlers for streams.
#![allow(unused)]

use crate::{peer::Manager, Context};
use quinn::{RecvStream, SendStream};
use std::sync::Arc;
use tokio::sync::RwLock;

mod ce128;
mod ce129;
mod ce131;
mod ce132;
mod ce133;
mod ce134;
mod ce135;
mod ce136;
mod ce137;
mod ce138;
mod ce139;
mod ce140;
mod ce141;
mod ce142;
mod ce143;
mod ce144;
mod ce145;
mod up0;

/// Handle an incoming stream.
pub async fn recv<C: Context>(
    send: SendStream,
    mut recv: RecvStream,
    context: Arc<C>,
    manager: Arc<RwLock<Manager>>,
) -> anyhow::Result<()> {
    let mut buf = [0u8; 1];
    recv.read_exact(&mut buf).await?;

    // Handle the stream.
    match buf[0] {
        0 => up0::recv(send, recv, context, manager).await,
        128 => ce128::recv(send, recv, context).await,
        129 => ce129::recv(send, recv, context).await,
        131 => ce131::recv(send, recv, context).await,
        132 => ce132::recv(send, recv, context).await,
        133 => ce133::recv(send, recv, context).await,
        134 => ce134::recv(send, recv, context).await,
        135 => ce135::recv(send, recv, context).await,
        136 => ce136::recv(send, recv, context).await,
        137 => ce137::recv(send, recv, context).await,
        138 => ce138::recv(send, recv, context).await,
        139 => ce139::recv(send, recv, context).await,
        140 => ce140::recv(send, recv, context).await,
        141 => ce141::recv(send, recv, context).await,
        142 => ce142::recv(send, recv, context).await,
        143 => ce143::recv(send, recv, context).await,
        144 => ce144::recv(send, recv, context).await,
        145 => ce145::recv(send, recv, context).await,
        unknown => anyhow::bail!("unknown stream type: {unknown}"),
    }
}
