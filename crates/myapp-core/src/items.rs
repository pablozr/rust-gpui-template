//! Use cases of the item list.

use myapp_domain::{ItemId, ItemList, ItemTitle};

use crate::{ItemStore, Problem, Result};

/// A change the person asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Append an item with this title.
    Add {
        /// The title as typed, not yet validated.
        title: String,
    },
    /// Flip the done flag of an item.
    Toggle(ItemId),
    /// Delete an item.
    Remove(ItemId),
}

/// Reads the saved list.
pub fn load(store: &dyn ItemStore) -> Result<ItemList> {
    let items = store.load()?;
    ItemList::from_items(items).map_err(|error| {
        tracing::warn!(%error, "saved items break a domain rule");
        Problem::StoreCorrupt
    })
}

/// Writes the list.
pub fn save(store: &dyn ItemStore, list: &ItemList) -> Result<()> {
    store.save(list.items())?;
    Ok(())
}

/// Applies a command and returns the id of the affected item. The list is unchanged on error.
pub fn apply(list: &mut ItemList, command: Command) -> Result<ItemId> {
    match command {
        Command::Add { title } => Ok(list.add(ItemTitle::parse(&title)?)),
        Command::Toggle(id) => {
            list.toggle(id)?;
            Ok(id)
        }
        Command::Remove(id) => Ok(list.remove(id)?.id()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DomainError, Item};
    use crate::fake::{Failure, MemoryStore};

    fn item(id: u64, title: &str) -> Item {
        Item::new(ItemId::new(id), ItemTitle::parse(title).unwrap(), false)
    }

    #[test]
    fn load_returns_what_the_store_holds() {
        let store = MemoryStore::with_items(vec![item(3, "three"), item(1, "one")]);
        let list = load(&store).unwrap();
        let ids: Vec<u64> = list.items().iter().map(|item| item.id().get()).collect();
        assert_eq!(ids, [3, 1]);
    }

    #[test]
    fn blank_title_is_rejected_and_the_list_is_unchanged() {
        let mut list = ItemList::default();
        let result = apply(&mut list, Command::Add { title: "  ".into() });
        assert_eq!(result, Err(Problem::Invalid(DomainError::EmptyTitle)));
        assert!(list.is_empty());
    }

    #[test]
    fn store_failure_becomes_a_retryable_problem() {
        let store = MemoryStore::default();
        store.fail_with(Some(Failure::Unavailable));
        let problem = save(&store, &ItemList::default()).unwrap_err();
        assert_eq!(problem, Problem::StoreUnavailable);
        assert!(problem.is_retryable());
        assert_eq!(store.save_count(), 0);
    }

    #[test]
    fn corrupt_store_is_reported_as_corrupt() {
        let store = MemoryStore::default();
        store.fail_with(Some(Failure::Corrupt));
        assert_eq!(load(&store).unwrap_err(), Problem::StoreCorrupt);

        let duplicated = MemoryStore::with_items(vec![item(1, "a"), item(1, "b")]);
        assert_eq!(load(&duplicated).unwrap_err(), Problem::StoreCorrupt);
    }
}
