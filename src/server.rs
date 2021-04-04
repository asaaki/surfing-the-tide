//! Example (backend) server

#![forbid(unsafe_code)]
#![warn(rust_2018_idioms)]
#![deny(warnings)]
#![deny(clippy::cargo)]
#![allow(clippy::cargo_common_metadata)]
#![deny(clippy::pedantic)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::panic)]

use opentelemetry_tide::TideExt;
use tide::{
    utils::{After, Before},
    Request, Response,
};

mod shared;

type MainResult = Result<(), Box<dyn std::error::Error>>;

const SVC_NAME: &str = env!("CARGO_CRATE_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

static DEFAULT_PORT: &str = "3000";
static DEFAULT_IP: &str = "127.0.0.1";

#[async_std::main]
async fn main() -> MainResult {
    shared::privdrop();
    tide::log::with_level(tide::log::LevelFilter::Warn);
    shared::init_global_propagator();
    let tracer = shared::jaeger_tracer(SVC_NAME, VERSION, "backend-123")?;

    let mut app = tide::new();
    app.with_middlewares(tracer, Some(shared::metrics_kvs()));

    app.with(After(|res: Response| async move {
        // dbg!(&res);
        Ok(res)
    }));
    app.with(Before(|req: Request<_>| async move {
        // dbg!(&req);
        req
    }));

    app.with(tide_compress::CompressMiddleware::new());
    // app.with(tide_delay::DelayMiddleware::new(std::time::Duration::from_millis(5)));

    app.at("/").get(|_| async move {
        Ok(format!(
            "Hello, OpenTelemetry! -- YYID: {}",
            yyid::Yyid::new()
        ))
    });

    eprintln!("Server started at {}", shared::addr());
    app.listen(shared::addr()).await?;
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
