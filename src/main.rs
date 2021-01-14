use opentelemetry::sdk::propagation::{
    BaggagePropagator, TextMapCompositePropagator, TraceContextPropagator,
};
use opentelemetry::{
    global,
    trace::{FutureExt, TraceContextExt},
    Context, KeyValue,
};
use opentelemetry_jaeger::Propagator as JaegerPropagator;
use opentelemetry_semantic_conventions::resource;
use tide::Request;

const SVC_NAME: &str = env!("CARGO_CRATE_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
include!(concat!(env!("OUT_DIR"), "/build_vars.rs"));

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tide::log::start();
    init_global_propagator();

    let tags = [
        resource::SERVICE_VERSION.string(VERSION),
        resource::SERVICE_INSTANCE_ID.string("surf-the-tide"),
        resource::PROCESS_EXECUTABLE_PATH
            .string(std::env::current_exe().unwrap().display().to_string()),
        resource::PROCESS_PID.string(std::process::id().to_string()),
        KeyValue::new("process.executable.profile", PROFILE),
    ];

    let (tracer, uninstall) = opentelemetry_jaeger::new_pipeline()
        .with_service_name(SVC_NAME)
        .with_tags(tags.iter().map(ToOwned::to_owned))
        .install()
        .expect("pipeline install failure");

    let otel_mw = opentelemetry_surf::OpenTelemetryTracingMiddleware::new(tracer.clone());
    let client = surf::client().with(otel_mw);

    let mut app = tide::with_state(client);

    app.with(opentelemetry_tide::OpenTelemetryTracingMiddleware::new(
        tracer.clone(),
        uninstall,
    ));

    app.at("/").get(|req: Request<surf::Client>| async move {
        // collect current tracing data, so we can pass it down
        let cx = Context::current();

        let client = req.state();
        let uri = "https://httpbin.org/headers";
        let surf_request = client.get(uri).build();

        let mut upstream_res = client.send(surf_request).with_context(cx.clone()).await?;
        let parsed_res = upstream_res.take_body().into_string().await?;
        let body = format!("upstream responded with:\n\n{}", parsed_res);
        Ok(body)
    });
    app.listen("127.0.0.1:3000").await?;

    Ok(())
}

pub fn init_global_propagator() {
    global::set_text_map_propagator(composite_propagator());
    // global::set_text_map_propagator(JaegerPropagator::new());
}

fn composite_propagator() -> TextMapCompositePropagator {
    let baggage_propagator = BaggagePropagator::new();
    let jaeger_propagator = JaegerPropagator::new(); // aka Uber headers
    let trace_context_propagator = TraceContextPropagator::new();

    TextMapCompositePropagator::new(vec![
        Box::new(jaeger_propagator),
        Box::new(baggage_propagator),
        Box::new(trace_context_propagator),
    ])
}
