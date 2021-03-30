use opentelemetry::{
    trace::{FutureExt, TraceContextExt},
    Context,
};
use tide::Request;

mod shared;

const SVC_NAME: &str = env!("CARGO_CRATE_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tide::log::with_level(tide::log::LevelFilter::Warn);
    shared::init_global_propagator();
    let tracer = shared::jaeger_tracer(SVC_NAME, VERSION, "surf-the-tide-9")?;

    let otel_mw = opentelemetry_surf::OpenTelemetryTracingMiddleware::new(tracer.clone());
    let client = create_client().with(otel_mw);

    let mut app = tide::with_state(client);

    app.with(opentelemetry_tide::OpenTelemetryTracingMiddleware::new(
        tracer.clone(),
    ));

    app.at("/").get(|req: Request<surf::Client>| async move {
        // collect current tracing data, so we can pass it down
        let cx = Context::current();
        let span = cx.span();

        let client = req.state();
        let uri = "http://localhost:3000/";
        let surf_request = client.get(uri).build();

        span.add_event("upstream.request.started".into(), vec![]);
        let mut upstream_res = client.send(surf_request).with_context(cx.clone()).await?;
        let upstream_body = upstream_res.take_body().into_string().await?;
        let body = format!("upstream responded with: \n{}", upstream_body);
        span.add_event("upstream.request.finished".into(), vec![]);
        Ok(body)
    });

    eprintln!("Don't forget to start a service on port 3000.");
    app.listen("0.0.0.0:4000").await?;
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}

// more custom http client setup: use isahc with metrics enabled
fn create_client() -> surf::Client {
    use http_client::isahc::IsahcClient;
    use isahc::prelude::*;

    let isahc = HttpClient::builder()
        .default_headers(&[("user-agent", "surf/isahc (with request metrics)")])
        .metrics(true)
        .build()
        .expect("isahc client could no be created");
    let http_client = IsahcClient::from_client(isahc);
    surf::Client::with_http_client(http_client)
}
