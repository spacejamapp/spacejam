//! Metrics http service.
#![cfg(feature = "metrics")]

use anyhow::Result;
use hyper::{body::Incoming, server::conn::http2, service::service_fn, Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use metrics::Metrics;
use std::convert::Infallible;
use tokio::net::TcpListener;

/// Serve the metrics.
pub async fn serve(addr: std::net::SocketAddr, metrics: Metrics) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("metrics server listening on {}", listener.local_addr()?);

    loop {
        let (stream, _) = listener.accept().await?;
        let metrics = metrics.clone();

        tokio::task::spawn(async move {
            let io = TokioIo::new(stream);
            if let Err(err) = http2::Builder::new(TokioExecutor::new())
                .serve_connection(
                    io,
                    service_fn(move |req| handle_request(req, metrics.clone())),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

/// Handle metrics requests
async fn handle_request(
    req: Request<Incoming>,
    metrics: Metrics,
) -> Result<Response<String>, Infallible> {
    if req.uri().path() == "/metrics" {
        let metrics_str = metrics
            .metrics()
            .unwrap_or_else(|_| "Error getting metrics".to_string());
        Ok(Response::new(metrics_str))
    } else {
        Ok(Response::new("404 Not Found".to_string()))
    }
}
