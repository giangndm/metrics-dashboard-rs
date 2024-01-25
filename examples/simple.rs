use std::time::{Duration, Instant};

use metrics::{counter, describe_counter, describe_gauge, gauge, Unit};
use metrics_dashboard::{build_dashboard_route, ChartType, DashboardOptions, HttpMetricMiddleware};
use poem::{
    get, handler, listener::TcpListener, middleware::Tracing, web::Path, EndpointExt, Route, Server,
};

#[handler]
fn hello(Path(name): Path<String>) -> String {
    format!("hello: {name}")
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

    let dashboard_options = DashboardOptions {
        custom_charts: vec![
            ChartType::Bar {
                metrics: vec![
                    "demo_metric2".to_string(),
                    "demo_metric3".to_string(),
                    "demo_metric4".to_string(),
                ],
                desc: "Demo metric bar".to_string(),
                unit: Unit::Count.as_canonical_label().to_string(),
            },
            ChartType::Line {
                metrics: vec![
                    "http_requests_total".to_string(),
                    "http_requests_errors".to_string(),
                ],
                desc: "Http requests".to_string(),
                unit: Unit::Count.as_canonical_label().to_string(),
            },
        ],
        include_default: true,
    };

    let app = Route::new()
        .at("/hello/:name", get(hello))
        .nest("/dashboard/", build_dashboard_route(dashboard_options))
        .with(HttpMetricMiddleware)
        .with(Tracing);

    tokio::spawn(async move {
        describe_gauge!("demo_live_time", Unit::Seconds, "Demo live time");
        let start = Instant::now();
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            gauge!("demo_live_time").set(start.elapsed());
        }
    });

    tokio::spawn(async move {
        describe_counter!("demo_metric2", "Demo metric2");
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;
            counter!("demo_metric2").increment(1);
        }
    });

    tokio::spawn(async move {
        describe_counter!("demo_metric3", "Demo metric3");
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;
            counter!("demo_metric3").increment(1);
        }
    });

    tokio::spawn(async move {
        describe_counter!("demo_metric4", "Demo metric4");
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;
            counter!("demo_metric4").increment(1);
        }
    });

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("hello-world")
        .run(app)
        .await
}
