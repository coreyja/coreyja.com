use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Context;
use opentelemetry_otlp::WithExportConfig;
use sentry::ClientInitGuard;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{
    layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter, Registry,
};
use tracing_tree::HierarchicalLayer;

pub fn setup_sentry() -> Option<ClientInitGuard> {
    let git_commit: Option<std::borrow::Cow<_>> =
        option_env!("VERGEN_GIT_SHA").map(std::convert::Into::into);
    let release_name =
        git_commit.unwrap_or_else(|| sentry::release_name!().unwrap_or_else(|| "dev".into()));

    if let Ok(sentry_dsn) = std::env::var("SENTRY_DSN") {
        println!("Sentry enabled");

        Some(sentry::init((
            sentry_dsn,
            sentry::ClientOptions {
                traces_sample_rate: 0.5,
                release: Some(release_name),
                ..Default::default()
            },
        )))
    } else {
        println!("Sentry not configured in this environment");

        None
    }
}

pub fn setup_tracing(crate_name: &str) -> color_eyre::Result<()> {
    let rust_log = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| format!("info,{crate_name}=trace,tower_http=debug,serenity=error"));

    let env_filter = EnvFilter::builder().parse(&rust_log).wrap_err_with(|| {
        color_eyre::eyre::eyre!("Couldn't create env filter from {}", rust_log)
    })?;

    let opentelemetry_layer = if let Ok(honeycomb_key) = std::env::var("HONEYCOMB_API_KEY") {
        let mut map = HashMap::<String, String>::new();
        map.insert("x-honeycomb-team".to_string(), honeycomb_key);
        map.insert("x-honeycomb-dataset".to_string(), "coreyja.com".to_string());

        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .http()
                    .with_endpoint("https://api.honeycomb.io/v1/traces")
                    .with_timeout(Duration::from_secs(3))
                    .with_headers(map),
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;

        let opentelemetry_layer = OpenTelemetryLayer::new(tracer);
        println!("Honeycomb layer configured");

        Some(opentelemetry_layer)
    } else {
        println!("Skipping Honeycomb layer");

        None
    };

    let hierarchical = {
        let hierarchical = HierarchicalLayer::default()
            .with_writer(std::io::stdout)
            .with_indent_lines(true)
            .with_indent_amount(2)
            .with_thread_names(true)
            .with_thread_ids(true)
            .with_verbose_exit(true)
            .with_verbose_entry(true)
            .with_targets(true);

        println!("Let's also log to stdout.");

        hierarchical
    };

    Registry::default()
        .with(hierarchical)
        .with(opentelemetry_layer)
        .with(env_filter)
        .with(sentry_tracing::layer())
        .try_init()?;

    Ok(())
}
