//! This crate provide simple auto-generate dashboard for [metric-rs](https://crates.io/crates/metrics) crate.
//! To intergrate to poem webserver, simple include to route like:
//!
//! ```rust
//! use metrics_dashboard::{build_dashboard_route, DashboardOptions, ChartType};
//! use poem::Route;
//!
//! let dashboard_options = DashboardOptions {
//!     charts: vec![
//!         ChartType::Line {
//!             metrics: vec![
//!                 "demo_live_time".to_string(),
//!                 "demo_live_time_max".to_string(),
//!             ],
//!             desc: Some("Demo metric line".to_string()),
//!         },
//!     ],
//!     include_default: true,
//! };
//!
//! let app = Route::new().nest("/dashboard/", build_dashboard_route(dashboard_options));
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
use std::vec;

#[cfg(feature = "system")]
use metrics_process::register_sysinfo_event;
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
use serde::{Deserialize, Serialize};

#[cfg(feature = "system")]
mod metrics_process;
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

#[derive(Debug, Clone, Default)]
pub struct DashboardOptions {
    pub charts: Vec<ChartType>,
    /// Whether to include metrics that not mention in the charts options.
    /// This is useful when you want to include all metrics in the dashboard.
    pub include_default: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", content = "meta")]
pub enum ChartType {
    Line {
        metrics: Vec<String>,
        desc: Option<String>,
    },
    Bar {
        metrics: Vec<String>,
        desc: Option<String>,
    },
}

#[derive(Debug, Serialize, Clone)]
pub struct ChartMeta {
    desc: Option<String>,
    keys: Vec<String>,
    chart_type: ChartType,
}

#[handler]
fn prometheus_metrics(Data(recorder): Data<&metrics_prometheus::Recorder<NoOp>>) -> String {
    prometheus::TextEncoder::new()
        .encode_to_string(&recorder.registry().gather())
        .expect("Should generate")
}

#[handler]
fn api_charts(Data(recorder): Data<&DashboardRecorder>) -> Json<Vec<ChartMeta>> {
    let option = &recorder.options;
    let mut res = vec![];
    for chart in option.charts.iter() {
        let (keys, desc) = match chart {
            ChartType::Line { metrics, desc } => (metrics.clone(), desc.clone()),
            ChartType::Bar { metrics, desc } => (metrics.clone(), desc.clone()),
        };
        let meta = ChartMeta {
            desc,
            keys,
            chart_type: chart.clone(),
        };
        res.push(meta);
    }
    if option.include_default {
        let metrics = recorder.metrics();
        for meta in metrics.iter() {
            let chart_type = ChartType::Line {
                metrics: vec![meta.key.clone()],
                desc: meta.desc.clone(),
            };
            let meta = ChartMeta {
                desc: meta.desc.clone(),
                keys: vec![meta.key.clone()],
                chart_type,
            };
            res.push(meta);
        }
    }

    Json(res)
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

pub fn build_dashboard_route(opts: DashboardOptions) -> Route {
    let recorder1 = metrics_prometheus::Recorder::builder()
        .with_failure_strategy(strategy::NoOp)
        .build();

    let recorder2 = DashboardRecorder::new(opts);

    let recoder_fanout = FanoutBuilder::default()
        .add_recorder(recorder1.clone())
        .add_recorder(recorder2.clone())
        .build();

    metrics::set_boxed_recorder(Box::new(recoder_fanout))
        .expect("Should register a recorder successfull");
    #[cfg(feature = "system")]
    register_sysinfo_event();

    let route = Route::new()
        .at("/prometheus", prometheus_metrics.data(recorder1))
        .at("/api/metrics", api_metrics.data(recorder2.clone()))
        .at("/api/charts", api_charts.data(recorder2.clone()))
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

#[allow(unused)]
pub(crate) fn round_up_f64_2digits(input: f64) -> f64 {
    (input * 100.0).round() / 100.0
}
