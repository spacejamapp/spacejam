//! Logger middleware

use jsonrpsee::{server::middleware::rpc::RpcServiceT, types::Request};

/// Logging service
#[derive(Clone)]
pub struct Logger<S>(pub S);

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
