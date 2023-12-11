//! This crate provide simple auto-generate dashboard for [metric-rs](https://crates.io/crates/metrics) crate.
//! To intergrate to poem webserver, simple include to route like:
//!
//! ```rust
//! use metrics_dashboard::build_dashboard_route;
//! use poem::Route;
//!
//! let app = Route::new().nest("/dashboard/", build_dashboard_route());
//! ```
//!
//! After init dashboard route, all of metrics defined metric will be exposed.
//!
//! ```rust
//! use metrics::{describe_counter, increment_counter};
//!
//! describe_counter!("demo_metric1", "Demo metric1");
//! increment_counter!("demo_metric1");
//! ```
use metrics_prometheus::failure::strategy::{self, NoOp};
use metrics_util::layers::FanoutBuilder;
pub use middleware::HttpMetricMiddleware;
use poem::EndpointExt;
use poem::{
    handler,
    web::{Data, Json, Query},
    Route,
};

#[cfg(not(feature = "embed"))]
use poem::endpoint::StaticFilesEndpoint;

#[cfg(feature = "embed")]
use poem::endpoint::{EmbeddedFileEndpoint, EmbeddedFilesEndpoint};
#[cfg(feature = "embed")]
use rust_embed::RustEmbed;

use recorder::{DashboardRecorder, MetricMeta, MetricValue};
use serde::Deserialize;

mod middleware;
mod recorder;

#[cfg(feature = "embed")]
#[derive(RustEmbed)]
#[folder = "public"]
pub struct Files;

#[derive(Debug, Deserialize)]
struct MetricQuery {
    keys: String,
}

#[handler]
fn prometheus_metrics(Data(recorder): Data<&metrics_prometheus::Recorder<NoOp>>) -> String {
    prometheus::TextEncoder::new()
        .encode_to_string(&recorder.registry().gather())
        .expect("Should generate")
}

#[handler]
fn api_metrics(Data(recorder): Data<&DashboardRecorder>) -> Json<Vec<MetricMeta>> {
    Json(recorder.metrics())
}

#[handler]
fn api_metrics_value(
    Data(recorder): Data<&DashboardRecorder>,
    Query(query): Query<MetricQuery>,
) -> Json<Vec<MetricValue>> {
    let keys = query.keys.split(';').collect::<Vec<&str>>();
    Json(recorder.metrics_value(keys))
}

pub fn build_dashboard_route() -> Route {
    let recorder1 = metrics_prometheus::Recorder::builder()
        .with_failure_strategy(strategy::NoOp)
        .build();

    let recorder2 = DashboardRecorder::new();

    let recoder_fanout = FanoutBuilder::default()
        .add_recorder(recorder1.clone())
        .add_recorder(recorder2.clone())
        .build();

    metrics::set_boxed_recorder(Box::new(recoder_fanout))
        .expect("Should register a recorder successfull");

    let route = Route::new()
        .at("/prometheus", prometheus_metrics.data(recorder1))
        .at("/api/metrics", api_metrics.data(recorder2.clone()))
        .at("/api/metrics_value", api_metrics_value.data(recorder2));

    #[cfg(not(feature = "embed"))]
    let route = route.nest(
        "/",
        StaticFilesEndpoint::new("./public/").index_file("index.html"),
    );

    #[cfg(feature = "embed")]
    let route = route.at("/", EmbeddedFileEndpoint::<Files>::new("index.html"));
    #[cfg(feature = "embed")]
    let route = route.nest("/", EmbeddedFilesEndpoint::<Files>::new());

    route
}
