//! Module for telemetry functionality such as logging

use anyhow::Result;
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Sets up logging. The log level is taken from the `RUST_LOG` env variable (default is `info`).
/// The logging format (pretty/json) is set by the `LOG_FORMAT` env variable.
pub fn setup_logging() -> Result<()> {
    let env_filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

    let format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());

    if format == "json" {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_writer(std::io::stderr), // so that we don't interfere with the std output
            )
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .pretty()
                    .with_writer(std::io::stderr), // so that we don't interfere with the std output
            )
            .init();
    }
    debug!("Debug mode is enabled. Sensitive data might be visible.");
    Ok(())
}
