//! Example front(end) server for testing

#![forbid(unsafe_code)]
#![warn(rust_2018_idioms)]
#![deny(warnings)]
#![deny(clippy::cargo)]
#![allow(clippy::cargo_common_metadata)]
#![deny(clippy::pedantic)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::panic)]

use opentelemetry::{
    trace::{FutureExt, TraceContextExt},
    Context, KeyValue,
};
use opentelemetry_surf::OpenTelemetryTracingMiddleware;
use opentelemetry_tide::TideExt;
use tide::Request;
use std::sync::atomic::{AtomicUsize, Ordering};
use async_std::sync::Arc;
use anyhow::{Context as ErrorContext, Result};

mod shared;

const SVC_NAME: &str = env!("CARGO_CRATE_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

static DEFAULT_PORT: &str = "4000";
static DEFAULT_IP: &str = "127.0.0.1";
static DEFAULT_UPSTREAM_URL: &str = "http://localhost:3000";

#[derive(Debug, Clone)]
struct State {
    client: surf::Client,
    upstream_urls: Vec<String>,
    counter: Arc<AtomicUsize>,
}

impl State {
    fn new(client: surf::Client, upstream_urls: Vec<String>) -> Self {
        let counter = Arc::new(AtomicUsize::new(0));

        Self { client, upstream_urls, counter }
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    shared::privdrop();
    tide::log::with_level(tide::log::LevelFilter::Warn);
    shared::init_global_propagator();
    let tracer = shared::jaeger_tracer(SVC_NAME, VERSION, "surf-the-tide-9")?;

    let otel_mw = OpenTelemetryTracingMiddleware::new(tracer.clone());
    let client = create_client().with(otel_mw);

    // let upstream_url = upstream_url();
    let upstream_urls = upstream_urls();
    let state = State::new(client, upstream_urls.clone());
    let mut app = tide::with_state(state);

    app.with_middlewares(tracer, Some(shared::metrics_kvs()));
    app.with(tide_compress::CompressMiddleware::new());

    app.at("/")
        .get(|req: Request<State>| async move {
            // collect current tracing data, so we can pass it down
            let cx = Context::current();
            let span = cx.span();

            let state = req.state();
            // each request increases this counter; overflow will result in wrap around
            let counter_value = state.counter.fetch_add(1, Ordering::Relaxed);

            let upstream_url = select_upstream(&state.upstream_urls, counter_value);
            let client = &state.client;
            let surf_request = client.get(upstream_url).build();

            span.add_event("upstream.request.started".into(), vec![]);
            let mut upstream_res = client.send(surf_request).with_context(cx.clone()).await?;
            let upstream_body = upstream_res.take_body().into_string().await?;
            let body = format!("upstream responded with: \n{}", upstream_body);
            span.add_event(
                "upstream.request.finished".into(),
                vec![KeyValue::new(
                    "upstream.body.length",
                    upstream_body.len().to_string(),
                )],
            );
            Ok(body)
        });
    let mut favicon_path = std::env::current_dir()?;
    favicon_path.push("static/favicon.ico");
    app.at("/favicon.ico").serve_file(&favicon_path).context(format!("favicon.ico not found at {}", &favicon_path.display()))?;

    eprintln!(
        "Don't forget to start an upstream service(s) on {:?}.",
        upstream_urls
    );
    eprintln!("Server started at {}", shared::addr());
    app.listen(shared::addr()).await?;
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

fn select_upstream<'a>(upstream_urls: &'a Vec<String>, counter_value: usize) -> &'a String {
    if upstream_urls.len() == 1 {
        return &upstream_urls[0];
    }
    // selection is simple round-robin based on provided upstream servers
    let index = counter_value % upstream_urls.len();
    &upstream_urls[index]
}

fn upstream_urls() -> Vec<String> {
    std::env::var("UPSTREAM_URLS")
        .map(|input| input.split(',').map(ToOwned::to_owned).collect() )
        .unwrap_or_else(|_| vec![upstream_url()])
}

fn upstream_url() -> String {
    std::env::var("UPSTREAM_URL").unwrap_or_else(|_| DEFAULT_UPSTREAM_URL.into())
}
