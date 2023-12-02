/// This crate provide simple auto-generate dashboard for [metric-rs](https://crates.io/crates/metrics) crate.
/// To intergrate to poem webserver, simple include to route like:
/// 
/// ```rust
/// use metrics_dashboard::build_dashboard_route;
/// use poem::Route;
/// 
/// let app = Route::new().nest("/dashboard/", build_dashboard_route());
/// ```

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

use recoder::{DashboardRecorder, MetricMeta, MetricValue};
use serde::Deserialize;

mod recoder;

#[cfg(feature = "embed")]
#[derive(RustEmbed)]
#[folder = "public"]
pub struct Files;

#[derive(Debug, Deserialize)]
struct MetricQuery {
    keys: String,
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
    let keys = query.keys.split(";").into_iter().collect::<Vec<&str>>();
    Json(recorder.metrics_value(keys))
}

pub fn build_dashboard_route() -> Route {
    let recorder = DashboardRecorder::new();
    metrics::set_boxed_recorder(Box::new(recorder.clone()))
        .expect("Should register a recorder successfull");

    let route = Route::new()
        .at("/api/metrics", api_metrics.data(recorder.clone()))
        .at("/api/metrics_value", api_metrics_value.data(recorder));

    #[cfg(not(feature = "embed"))]
    let route = route.nest("/", StaticFilesEndpoint::new("./public/").index_file("index.html"));

    #[cfg(feature = "embed")]
    let route = route.at("/", EmbeddedFileEndpoint::<Files>::new("index.html"));
    #[cfg(feature = "embed")]
    let route = route.nest("/", EmbeddedFilesEndpoint::<Files>::new());
    route
}
