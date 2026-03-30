//! Error handling extensions and utilities

use std::fmt;

/// Extension trait for Results to add context to errors
pub trait ErrorContext<T> {
    /// Add context to an error
    fn context<C>(self, context: C) -> Result<T, crate::Error>
    where
        C: fmt::Display + Send + Sync + 'static;

    /// Add context with a closure (lazy evaluation)
    fn with_context<C, F>(self, f: F) -> Result<T, crate::Error>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: Into<crate::Error>,
{
    fn context<C>(self, context: C) -> Result<T, crate::Error>
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let err = e.into();
            tracing::error!(error = ?err, context = %context, "Operation failed");
            err
        })
    }

    fn with_context<C, F>(self, f: F) -> Result<T, crate::Error>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| {
            let err = e.into();
            let context = f();
            tracing::error!(error = ?err, context = %context, "Operation failed");
            err
        })
    }
}

/// Log and convert an error
#[macro_export]
macro_rules! log_error {
    ($err:expr) => {{
        tracing::error!(error = ?$err, "Error occurred");
        $err
    }};
    ($err:expr, $msg:expr) => {{
        tracing::error!(error = ?$err, $msg);
        $err
    }};
    ($err:expr, $msg:expr, $($key:tt = $value:expr),+) => {{
        tracing::error!(error = ?$err, $($key = $value,)+ $msg);
        $err
    }};
}

/// Create an error with context
#[macro_export]
macro_rules! error_with_context {
    ($msg:expr) => {
        crate::Error::Other(anyhow::anyhow!($msg))
    };
    ($fmt:expr, $($arg:tt)*) => {
        crate::Error::Other(anyhow::anyhow!($fmt, $($arg)*))
    };
}

/// Early return with error logging
#[macro_export]
macro_rules! bail_with_log {
    ($msg:expr) => {{
        let err = $crate::error_with_context!($msg);
        tracing::error!(error = ?err, "Bailing with error");
        return Err(err);
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        let err = $crate::error_with_context!($fmt, $($arg)*);
        tracing::error!(error = ?err, "Bailing with error");
        return Err(err);
    }};
}
