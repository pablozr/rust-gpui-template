//! An in-memory `ItemStore` for tests.

use std::io;
use std::sync::{Mutex, MutexGuard, PoisonError};

use myapp_domain::Item;

use crate::{ItemStore, StoreError};

/// How the fake store should fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Failure {
    /// Behave like a medium that cannot be reached.
    Unavailable,
    /// Behave like a medium holding unreadable data.
    Corrupt,
}

/// Keeps items in memory.
#[derive(Debug, Default)]
pub struct MemoryStore {
    items: Mutex<Vec<Item>>,
    failure: Mutex<Option<Failure>>,
}

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl MemoryStore {
    /// A store that already holds `items`.
    pub fn with_items(items: Vec<Item>) -> Self {
        Self {
            items: Mutex::new(items),
            ..Self::default()
        }
    }

    /// Makes every following `load` and `save` fail, or succeed again with `None`.
    pub fn fail_with(&self, failure: Option<Failure>) {
        *locked(&self.failure) = failure;
    }

    /// The items last saved.
    pub fn saved(&self) -> Vec<Item> {
        locked(&self.items).clone()
    }

    fn check(&self) -> Result<(), StoreError> {
        match *locked(&self.failure) {
            None => Ok(()),
            Some(Failure::Unavailable) => Err(StoreError::Io(io::Error::other("fake"))),
            Some(Failure::Corrupt) => Err(StoreError::Corrupt("fake".into())),
        }
    }
}

impl ItemStore for MemoryStore {
    fn load(&self) -> Result<Vec<Item>, StoreError> {
        self.check()?;
        Ok(self.saved())
    }

    fn save(&self, items: &[Item]) -> Result<(), StoreError> {
        self.check()?;
        *locked(&self.items) = items.to_vec();
        Ok(())
    }
}
