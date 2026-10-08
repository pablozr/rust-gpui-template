//! Domain types and rules. No I/O, no async, no UI.

mod item;

pub use item::{DomainError, Item, ItemId, ItemList, ItemTitle, MAX_TITLE_CHARS};
