//! Library facade for integration tests.
//!
//! `cargo test` builds a separate binary per test file in `tests/`.
//! Those binaries can't reach into the `gist` binary's modules directly,
//! so we expose them here.

pub mod extractors;
pub mod format;
pub mod output;
pub mod source;
