use std::time::Duration;

use metrics::{describe_counter, increment_counter};
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
        .nest("/dashboard/", build_dashboard_route())
        .with(HttpMetricMiddleware)
        .with(Tracing);

    tokio::spawn(async move {
        describe_counter!("demo_metric1", "Demo metric1");
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            increment_counter!("demo_metric1");
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
