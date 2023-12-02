use std::time::Instant;

use poem::{Endpoint, IntoResponse, Middleware, Request, Response, Result};

/// Middleware for [`tracing`](https://crates.io/crates/tracing).
#[derive(Default)]
pub struct HttpMetricMiddleware;

impl<E: Endpoint> Middleware<E> for HttpMetricMiddleware {
    type Output = HttpMetricMiddlewareEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        HttpMetricMiddlewareEndpoint { inner: ep }
    }
}

/// Endpoint for `Tracing` middleware.
pub struct HttpMetricMiddlewareEndpoint<E> {
    inner: E,
}

#[async_trait::async_trait]
impl<E: Endpoint> Endpoint for HttpMetricMiddlewareEndpoint<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        let now = Instant::now();
        let res = self.inner.call(req).await;
        let latency = now.elapsed();

        match res {
            Ok(resp) => {
                let resp = resp.into_response();
                metrics::increment_counter!("http_requests_total");
                metrics::histogram!("http_requests_duration_seconds", latency.as_secs_f64());
                Ok(resp)
            }
            Err(err) => {
                metrics::increment_counter!("http_requests_error");
                Err(err)
            }
        }
    }
}
