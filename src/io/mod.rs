//! The `io` module exposes structs and utilities to read and write to files
//! that are eligable for licensing.

mod document;
mod template_cache;
mod tree;

pub use document::*;
pub use tree::*;
