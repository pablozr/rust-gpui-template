//! Use cases and ports. Synchronous: callers decide which thread runs them; I/O only behind ports.

pub use myapp_domain as domain;

pub mod items;
mod problem;
mod store;

#[cfg(any(test, feature = "test-support"))]
pub mod fake;

pub use problem::{Problem, Result};
pub use store::{ItemStore, StoreError};
