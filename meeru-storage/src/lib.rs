//! Meeru Storage - Data persistence layer
//!
//! This crate handles all data storage operations including SQLite database
//! management, file storage for email content, and search indexing.

pub mod database;
pub mod error;
pub mod migrations;

pub use error::{Error, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
