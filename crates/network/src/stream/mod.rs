//! Streams for the network.
//!
//! Functional handlers for streams.

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

impl<C: runtime::Config> Network<C> {
    /// Handle an incoming stream.
    #[tracing::instrument(skip_all, level = "debug", fields(peer = ?peer.to_string()), name = "stream")]
    pub async fn handle(&self, peer: PeerId, send: SendStream, mut recv: RecvStream) {
        let mut buf = [0; 1];
        if let Err(e) = recv.read_exact(&mut buf).await {
            tracing::debug!("failed to read stream type: {e:?}");
        }

        let bufs = match buf[0] {
            0 => "up0".into(),
            n => format!("ce{n}"),
        };

        if let Err(e) = match buf[0] {
            0 => self.recv_up0(peer, send, recv).await,
            128 => self.recv_ce128(send, recv).await,
            129 => self.recv_ce129(send, recv).await,
            131 => self.recv_ce131(send, recv).await,
            132 => self.recv_ce132(send, recv).await,
            133 => self.recv_ce133(send, recv).await,
            134 => self.recv_ce134(send, recv).await,
            135 => self.recv_ce135(send, recv).await,
            136 => self.recv_ce136(send, recv).await,
            137 => self.recv_ce137(send, recv).await,
            138 => self.recv_ce138(send, recv).await,
            139 => self.recv_ce139(send, recv).await,
            140 => self.recv_ce140(send, recv).await,
            141 => self.recv_ce141(send, recv).await,
            142 => self.recv_ce142(send, recv).await,
            143 => self.recv_ce143(send, recv).await,
            144 => self.recv_ce144(send, recv).await,
            145 => self.recv_ce145(send, recv).await,
            unknown => Err(anyhow::anyhow!("unknown stream type: {unknown}")),
        } {
            tracing::warn!("{bufs}: {e:?}");
        }
    }
}

pub mod ext {
    use quinn::{RecvStream, SendStream};
    use serde::{de::DeserializeOwned, Serialize};

    #[allow(unused)]
    /// Write extension trait for `SendStream`
    pub trait Write {
        /// Write a message to the stream.
        async fn write(&self, stream: &mut SendStream) -> anyhow::Result<()>;
    }

    impl<T: Serialize> Write for T {
        async fn write(&self, stream: &mut SendStream) -> anyhow::Result<()> {
            let encoded = codec::encode(&self)?;
            let length = encoded.len() as u32;
            stream.write(&length.to_le_bytes()).await?;
            stream.write(&encoded).await?;
            Ok(())
        }
    }
    /// Read extension trait for `RecvStream`
    pub trait Read: Sized {
        /// Read a message from the stream.
        async fn read(recv: &mut RecvStream) -> anyhow::Result<Self>;
    }

    impl<T: DeserializeOwned> Read for T {
        async fn read(recv: &mut RecvStream) -> anyhow::Result<Self> {
            let mut buf = [0; 4];
            recv.read_exact(&mut buf).await?;
            let length = u32::from_le_bytes(buf) as usize;

            let mut buf = vec![0; length];
            recv.read_exact(&mut buf).await?;
            let data: Self = codec::decode(&buf)?;
            Ok(data)
        }
    }
}
