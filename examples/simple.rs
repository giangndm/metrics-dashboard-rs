use std::time::{Duration, Instant};

use metrics::{describe_counter, describe_gauge, gauge, increment_counter, Unit};
use metrics_dashboard::{build_dashboard_route, HttpMetricMiddleware};
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

    let app = Route::new()
        .at("/hello/:name", get(hello))
        .nest(
            "/dashboard/",
            build_dashboard_route(vec![("demo_live_time", "demo_live_time_max")]),
        )
        .with(HttpMetricMiddleware)
        .with(Tracing);

    tokio::spawn(async move {
        describe_gauge!("demo_live_time", Unit::Seconds, "Demo live time");
        let start = Instant::now();
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            gauge!("demo_live_time", start.elapsed().as_secs_f64());
        }
    });

    tokio::spawn(async move {
        describe_gauge!("demo_live_time_max", Unit::Seconds, "Demo live time max");
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            gauge!("demo_live_time_max", 10.0);
        }
    });

    tokio::spawn(async move {
        describe_counter!("demo_metric2", "Demo metric2");
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;
            increment_counter!("demo_metric2");
        }
    });

    tokio::spawn(async move {
        describe_counter!("demo_metric3", "Demo metric3");
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;
            increment_counter!("demo_metric3");
        }
    });

    tokio::spawn(async move {
        describe_counter!("demo_metric4", "Demo metric4");
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;
            increment_counter!("demo_metric4");
        }
    });

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("hello-world")
        .run(app)
        .await
}
