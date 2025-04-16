//! The offchain components of SpaceJam

use ::metrics::Metrics;
use ::rpc::{middleware::rpc::RpcServiceT, types::Request, ApiServer, RpcServiceBuilder, Server};
use rpc::Rpc;
use runtime::Runtime;
use std::{net::SocketAddr, sync::Arc};

mod metrics;
mod rpc;

/// The entypoint of the offchain services
pub struct Offchain<C: runtime::Config> {
    /// The RPC server
    pub rpc: Rpc<C>,
}

impl<C: runtime::Config> Offchain<C> {
    /// Create a new offchain component
    pub fn new(runtime: Arc<Runtime<C>>) -> Self {
        Self {
            rpc: Rpc::new(runtime),
        }
    }

    /// Start the offchain services
    pub async fn start(
        self,
        rpc: SocketAddr,
        metrics: Metrics,
        metrics_addr: SocketAddr,
    ) -> anyhow::Result<()> {
        let rpc_middleware = RpcServiceBuilder::new().layer_fn(Logger);
        let server = Server::builder()
            .set_rpc_middleware(rpc_middleware)
            .build(rpc)
            .await?;

        let addr = server.local_addr()?;
        tracing::info!("Listening RPC on {}", addr);

        tokio::select! {
            _ = server.start(self.rpc.into_rpc()).stopped() => {}
            _ = metrics::serve(metrics_addr, metrics) => {}
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Logger<S>(S);

impl<'a, S> RpcServiceT<'a> for Logger<S>
where
    S: RpcServiceT<'a> + Send + Sync,
{
    type Future = S::Future;

    #[tracing::instrument(skip_all, name = "jsonrpc", fields(method = req.method.to_string()))]
    fn call(&self, req: Request<'a>) -> Self::Future {
        tracing::debug!("{:?}", req.params);
        self.0.call(req)
    }
}
