//! The port through which the core reads and writes items.

use myapp_domain::Item;

/// Where items are kept. Implemented by the storage adapter and by `fake::MemoryStore`.
pub trait ItemStore: Send + Sync {
    /// Reads every saved item; an empty store yields an empty list.
    fn load(&self) -> std::result::Result<Vec<Item>, StoreError>;

    /// Replaces the saved items.
    fn save(&self, items: &[Item]) -> std::result::Result<(), StoreError>;
}

/// Why a store could not read or write.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// The underlying medium could not be reached.
    #[error("could not access the item store")]
    Io(#[from] std::io::Error),
    /// The medium was reached but its content cannot be understood.
    #[error("the item store holds data this version cannot read")]
    Corrupt(#[source] Box<dyn std::error::Error + Send + Sync>),
}
