use std::time::Instant;

use poem::{
    web::RealIp, Endpoint, FromRequest, IntoResponse, Middleware, PathPattern, Request, Response,
};
use tracing::{Instrument, Level};

#[derive(Default)]
pub struct Tracing;

impl<E: Endpoint> Middleware<E> for Tracing {
    type Output = TracingEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        TracingEndpoint { inner: ep }
    }
}

pub struct TracingEndpoint<E> {
    inner: E,
}

impl<E: Endpoint> Endpoint for TracingEndpoint<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> poem::Result<Self::Output> {
        let remote_addr = RealIp::from_request_without_body(&req)
            .await
            .ok()
            .and_then(|real_ip| real_ip.0)
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| req.remote_addr().to_string());

        let span = tracing::span!(
            target: module_path!(),
            Level::INFO,
            "request",
            remote_addr = %remote_addr,
            version = ?req.version(),
            method = %req.method(),
            uri = %req.original_uri(),
        );

        if let Some(path_pattern) = req.data::<PathPattern>() {
            span.record("path_pattern", path_pattern.0.as_ref());
        }

        async move {
            let now = Instant::now();
            let res = self.inner.call(req).await;
            let duration = now.elapsed();

            match res {
                Ok(resp) => {
                    let resp = resp.into_response();
                    let status = resp.status();
                    if status.is_server_error() {
                        tracing::error!(
                            %status,
                            duration = ?duration,
                            "response"
                        );
                    } else if status.is_client_error() {
                        tracing::warn!(
                            %status,
                            duration = ?duration,
                            "response"
                        );
                    } else {
                        tracing::debug!(
                            %status,
                            duration = ?duration,
                            "response"
                        );
                    }
                    Ok(resp)
                }
                Err(error) => {
                    let status = error.status();
                    if status.is_server_error() {
                        tracing::error!(
                            %status,
                            ?error,
                            duration = ?duration,
                            "error"
                        );
                    } else if status.is_client_error() {
                        tracing::warn!(
                            %status,
                            ?error,
                            duration = ?duration,
                            "error"
                        );
                    } else {
                        tracing::debug!(
                            %status,
                            ?error,
                            duration = ?duration,
                            "error"
                        );
                    }
                    Err(error)
                }
            }
        }
        .instrument(span)
        .await
    }
}
