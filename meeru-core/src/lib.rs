//! Meeru Core - The business logic layer for the Meeru email client
//!
//! This crate contains the core functionality for managing email accounts,
//! synchronization, and the unified folder system.

pub mod account;
pub mod email;
pub mod error;
pub mod logging;
pub mod sync;
pub mod unified;
pub mod utils;

pub use error::{Error, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
