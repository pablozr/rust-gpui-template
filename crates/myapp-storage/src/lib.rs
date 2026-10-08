//! Adapters that implement the core's ports. Wired only in the app's `main.rs`.

mod json_file;

pub use json_file::JsonFileStore;
