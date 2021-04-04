#![doc(hidden)]

use opentelemetry::sdk::trace::{self, Config, Tracer};
use opentelemetry::{global, trace::TraceError, KeyValue};
use opentelemetry_jaeger::Propagator as JaegerPropagator;
use opentelemetry_semantic_conventions::resource;
use std::{env::var, net::SocketAddr};

include!(concat!(env!("OUT_DIR"), "/build_vars.rs"));

#[cfg(target_os = "linux")]
#[inline]
pub fn privdrop() {
    if nix::unistd::Uid::effective().is_root() {
        privdrop::PrivDrop::default()
            .chroot("/var/empty")
            .user("nobody")
            .apply()
            .unwrap_or_else(|e| panic!("Failed to drop privileges: {}", e));
    }
}

#[cfg(not(target_os = "linux"))]
#[inline]
pub fn privdrop() {
    /* noop */
}

pub fn addr() -> SocketAddr {
    format!("{}:{}", host_ip(), port())
        .parse()
        .expect("HOST_IP:PORT does not form a valid address")
}
fn host_ip() -> String {
    var("HOST_IP").unwrap_or_else(|_| super::DEFAULT_IP.into())
}
fn port() -> String {
    var("PORT").unwrap_or_else(|_| super::DEFAULT_PORT.into())
}

pub fn init_global_propagator() {
    global::set_text_map_propagator(JaegerPropagator::new());
}

#[allow(dead_code)]
pub fn trace_config() -> Config {
    trace::config()
    // can accept more options like:
    // .with_sampler(Sampler::TraceIdRatioBased(0.2))
}

pub fn jaeger_tracer(
    svc_name: &str,
    version: &str,
    instance_id: &str,
) -> Result<Tracer, TraceError> {
    // https://github.com/open-telemetry/opentelemetry-specification/tree/main/specification/resource/semantic_conventions
    let tags = [
        resource::HOST_NAME
            .string(hostname()),
        resource::SERVICE_VERSION.string(version.to_owned()),
        resource::SERVICE_INSTANCE_ID.string(instance_id.to_owned()),
        resource::PROCESS_EXECUTABLE_PATH
            .string(std::env::current_exe().unwrap().display().to_string()),
        resource::PROCESS_PID.string(std::process::id().to_string()),
        KeyValue::new("process.executable.profile", PROFILE),
    ];

    opentelemetry_jaeger::new_pipeline()
        .with_service_name(svc_name)
        .with_tags(tags.iter().map(ToOwned::to_owned))
        .install_batch(opentelemetry::runtime::AsyncStd)
}

pub fn metrics_kvs() -> Vec<KeyValue> {
    vec![
        KeyValue::new("hostname", hostname())
    ]
}

fn hostname() -> String {
    std::env::var("HOSTNAME").unwrap_or_else(|_| "NO_HOSTNAME_SET".into())
}
