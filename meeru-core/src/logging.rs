//! Logging configuration and utilities

use tracing::Level;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize logging for the application
pub fn init_logging() {
    init_logging_with_level(None);
}

/// Initialize logging with a specific level
pub fn init_logging_with_level(level: Option<Level>) {
    let env_filter = match level {
        Some(l) => EnvFilter::new(format!("meeru={}", l))
            .add_directive("meeru_core=debug".parse().unwrap())
            .add_directive("meeru_storage=debug".parse().unwrap())
            .add_directive("meeru_providers=debug".parse().unwrap()),
        None => EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("meeru=info")),
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true).with_thread_ids(true))
        .with(env_filter)
        .init();
}

/// Initialize test logging (single init, less verbose)
#[cfg(test)]
pub fn init_test_logging() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let env_filter = EnvFilter::new("meeru=debug,test=debug");

        tracing_subscriber::registry()
            .with(fmt::layer().with_test_writer().with_target(false))
            .with(env_filter)
            .init();
    });
}

/// Log a message with structured fields
#[macro_export]
macro_rules! log_with_context {
    ($level:expr, $msg:expr, $($key:tt = $value:expr),+) => {
        match $level {
            tracing::Level::ERROR => tracing::error!($($key = $value,)+ $msg),
            tracing::Level::WARN => tracing::warn!($($key = $value,)+ $msg),
            tracing::Level::INFO => tracing::info!($($key = $value,)+ $msg),
            tracing::Level::DEBUG => tracing::debug!($($key = $value,)+ $msg),
            tracing::Level::TRACE => tracing::trace!($($key = $value,)+ $msg),
        }
    };
}
