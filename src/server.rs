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
use rand::distributions::{Bernoulli, Distribution, Uniform};
use std::time::Duration;
use tide::{
    utils::{After, Before},
    Request, Response, Result as TideResult,
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
    let mut app = tide::with_state(random_waits());
    app.with_middlewares(tracer, shared::metrics_config());
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

    app.at("/").get(handler);

    eprintln!("Server started at {}", shared::addr());
    app.listen(shared::addr()).await?;
    opentelemetry::global::force_flush_tracer_provider();
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}

async fn handler(req: Request<bool>) -> TideResult {
    let duration = random_duration(*req.state());
    if duration.as_nanos() > 0 {
        async_std::task::sleep(duration).await;
        Ok(format!(
            "Hey, I am a 'slow' response, waited for {} microseconds",
            duration.as_micros()
        )
        .into())
    } else {
        Ok(format!("Hello, OpenTelemetry! -- YYID: {}", yyid::Yyid::new()).into())
    }
}

fn random_duration(random_waits: bool) -> Duration {
    if !random_waits {
        return Duration::from_secs(0);
    };

    let mut rng = rand::thread_rng();
    let trigger = Bernoulli::new(0.25).expect("bernoulli failed"); // change of 25 % to be true
    let trigger = trigger.sample(&mut rng);
    if trigger {
        let wait_micros: Uniform<u64> = Uniform::new_inclusive(42, 125_000);
        let wait_micros = wait_micros.sample(&mut rng);
        Duration::from_micros(wait_micros)
    } else {
        Duration::from_secs(0)
    }
}

fn random_waits() -> bool {
    let truthies = ["1", "true", "TRUE", "yes", "YES", "Y", "on", "ON"];
    std::env::var("RANDOM_WAITS")
        .map(|v| truthies.iter().any(|m| &v == m))
        .unwrap_or(false)
}
