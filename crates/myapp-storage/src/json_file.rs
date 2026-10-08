//! Items kept in one JSON file.

use std::io::{ErrorKind, Write as _};
use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};
use std::{fs, io};

use myapp_core::domain::{Item, ItemId, ItemTitle};
use myapp_core::{ItemStore, StoreError};
use serde::{Deserialize, Serialize};

const VERSION: u32 = 1;

#[derive(Deserialize)]
struct StoredFile {
    version: u32,
    items: Vec<StoredItem>,
}

#[derive(Deserialize)]
struct StoredItem {
    id: u64,
    title: String,
    done: bool,
}

#[derive(Serialize)]
struct StoredFileRef<'a> {
    version: u32,
    items: Vec<StoredItemRef<'a>>,
}

#[derive(Serialize)]
struct StoredItemRef<'a> {
    id: u64,
    title: &'a str,
    done: bool,
}

impl<'a> From<&'a Item> for StoredItemRef<'a> {
    fn from(item: &'a Item) -> Self {
        Self {
            id: item.id().get(),
            title: item.title().as_str(),
            done: item.is_done(),
        }
    }
}

impl TryFrom<StoredItem> for Item {
    type Error = StoreError;

    fn try_from(stored: StoredItem) -> Result<Self, StoreError> {
        let title =
            ItemTitle::parse(&stored.title).map_err(|error| StoreError::Corrupt(error.into()))?;
        Ok(Item::new(ItemId::new(stored.id), title, stored.done))
    }
}

/// Keeps the items in a JSON file, replaced as a whole on every save.
#[derive(Debug)]
pub struct JsonFileStore {
    path: PathBuf,
    /// Saves share one temporary file, so they run one at a time.
    writing: Mutex<()>,
}

impl JsonFileStore {
    /// A store backed by the file at `path`; the file is created on the first save.
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            writing: Mutex::new(()),
        }
    }
}

impl ItemStore for JsonFileStore {
    fn load(&self) -> Result<Vec<Item>, StoreError> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.into()),
        };
        let file: StoredFile =
            serde_json::from_slice(&bytes).map_err(|error| StoreError::Corrupt(error.into()))?;
        if file.version != VERSION {
            return Err(StoreError::Corrupt(
                format!("unsupported file version {}", file.version).into(),
            ));
        }
        file.items.into_iter().map(Item::try_from).collect()
    }

    fn save(&self, items: &[Item]) -> Result<(), StoreError> {
        let file = StoredFileRef {
            version: VERSION,
            items: items.iter().map(StoredItemRef::from).collect(),
        };
        let bytes = serde_json::to_vec_pretty(&file)
            .map_err(|error| StoreError::Io(io::Error::other(error)))?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let _writing = self.writing.lock().unwrap_or_else(PoisonError::into_inner);
        let temporary = self.path.with_extension("json.tmp");
        let mut file = fs::File::create(&temporary)?;
        file.write_all(&bytes)?;
        // The data must be on disk before the rename, or a crash could leave the new name
        // pointing at an empty file. The rename itself replaces the old file atomically.
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, &self.path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: u64, title: &str, done: bool) -> Item {
        Item::new(ItemId::new(id), ItemTitle::parse(title).unwrap(), done)
    }

    fn store_in(directory: &tempfile::TempDir) -> JsonFileStore {
        JsonFileStore::new(directory.path().join("nested").join("items.json"))
    }

    #[test]
    fn saved_items_load_back_unchanged() {
        let directory = tempfile::tempdir().unwrap();
        let store = store_in(&directory);
        let items = vec![item(2, "second", true), item(1, "first", false)];
        store.save(&items).unwrap();
        assert_eq!(store.load().unwrap(), items);
    }

    #[test]
    fn missing_file_means_no_items() {
        let directory = tempfile::tempdir().unwrap();
        assert_eq!(store_in(&directory).load().unwrap(), []);
    }

    #[test]
    fn invalid_json_is_corrupt() {
        let directory = tempfile::tempdir().unwrap();
        let store = store_in(&directory);
        store.save(&[]).unwrap();
        fs::write(&store.path, b"{ not json").unwrap();
        assert!(matches!(store.load(), Err(StoreError::Corrupt(_))));
    }

    #[test]
    fn unknown_version_is_corrupt() {
        let directory = tempfile::tempdir().unwrap();
        let store = store_in(&directory);
        store.save(&[]).unwrap();
        fs::write(&store.path, br#"{"version": 2, "items": []}"#).unwrap();
        assert!(matches!(store.load(), Err(StoreError::Corrupt(_))));
    }

    #[test]
    fn saves_from_several_threads_never_corrupt_the_file() {
        let directory = tempfile::tempdir().unwrap();
        let store = store_in(&directory);
        std::thread::scope(|scope| {
            for id in 1..=8 {
                let store = &store;
                scope.spawn(move || {
                    for _ in 0..25 {
                        store.save(&[item(id, "same", false)]).unwrap();
                    }
                });
            }
        });
        assert_eq!(store.load().unwrap().len(), 1);
    }
}
