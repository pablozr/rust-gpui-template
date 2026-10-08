//! The item list and the rules that keep it valid.

use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

/// Longest title, in characters.
pub const MAX_TITLE_CHARS: usize = 80;

/// Why a domain rule rejected an input.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DomainError {
    /// The title is empty after trimming.
    #[error("the title is empty")]
    EmptyTitle,
    /// The title has more than [`MAX_TITLE_CHARS`] characters.
    #[error("the title is longer than {MAX_TITLE_CHARS} characters")]
    TitleTooLong,
    /// No item has this id.
    #[error("no item with id {0}")]
    UnknownItem(ItemId),
    /// Every id has been handed out.
    #[error("no item ids are left")]
    NoIdsLeft,
}

/// Why saved items cannot become a list.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RestoreError {
    /// Two items share this id.
    #[error("more than one item with id {0}")]
    DuplicateItem(ItemId),
    /// The id is the largest possible one, so no id can follow it.
    #[error("item id {0} leaves no room for new ids")]
    IdOutOfRange(ItemId),
}

/// Identifies an item for as long as it exists. Never reused within a list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemId(u64);

impl ItemId {
    /// Wraps a raw id, e.g. one read from storage.
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// The raw id.
    pub fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A non-blank title of at most [`MAX_TITLE_CHARS`] characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemTitle(Arc<str>);

impl ItemTitle {
    /// Trims `text` and checks it against the title rules.
    pub fn parse(text: &str) -> Result<Self, DomainError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyTitle);
        }
        if trimmed.chars().count() > MAX_TITLE_CHARS {
            return Err(DomainError::TitleTooLong);
        }
        Ok(Self(trimmed.into()))
    }

    /// The title text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The title text, shared without copying.
    pub fn to_shared(&self) -> Arc<str> {
        Arc::clone(&self.0)
    }
}

/// One entry of the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    id: ItemId,
    title: ItemTitle,
    done: bool,
}

impl Item {
    /// Builds an item from already validated parts.
    pub fn new(id: ItemId, title: ItemTitle, done: bool) -> Self {
        Self { id, title, done }
    }

    /// The item's id.
    pub fn id(&self) -> ItemId {
        self.id
    }

    /// The item's title.
    pub fn title(&self) -> &ItemTitle {
        &self.title
    }

    /// Whether the item is marked as done.
    pub fn is_done(&self) -> bool {
        self.done
    }
}

/// Items in display order, with unique ids.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemList {
    items: Vec<Item>,
    next_id: u64,
}

impl ItemList {
    /// Restores a list, e.g. from storage. New ids continue after the highest one.
    pub fn from_items(items: Vec<Item>) -> Result<Self, RestoreError> {
        let mut seen = HashSet::new();
        let mut next_id = 0;
        for item in &items {
            if !seen.insert(item.id) {
                return Err(RestoreError::DuplicateItem(item.id));
            }
            let after = item.id.get().checked_add(1);
            next_id = next_id.max(after.ok_or(RestoreError::IdOutOfRange(item.id))?);
        }
        Ok(Self { items, next_id })
    }

    /// Appends an undone item and returns its id.
    pub fn add(&mut self, title: ItemTitle) -> Result<ItemId, DomainError> {
        let next_id = self.next_id.checked_add(1).ok_or(DomainError::NoIdsLeft)?;
        let id = ItemId::new(self.next_id);
        self.next_id = next_id;
        self.items.push(Item::new(id, title, false));
        Ok(id)
    }

    /// Flips the done flag of an item.
    pub fn toggle(&mut self, id: ItemId) -> Result<(), DomainError> {
        let position = self.position(id).ok_or(DomainError::UnknownItem(id))?;
        self.items[position].done = !self.items[position].done;
        Ok(())
    }

    /// Removes an item and returns it.
    pub fn remove(&mut self, id: ItemId) -> Result<Item, DomainError> {
        let position = self.position(id).ok_or(DomainError::UnknownItem(id))?;
        Ok(self.items.remove(position))
    }

    /// The items in display order.
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    /// Where an item sits in display order.
    pub fn position(&self, id: ItemId) -> Option<usize> {
        self.items.iter().position(|item| item.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn title(text: &str) -> ItemTitle {
        ItemTitle::parse(text).unwrap()
    }

    fn item(id: u64) -> Item {
        Item::new(ItemId::new(id), title("item"), false)
    }

    #[test]
    fn title_is_trimmed_and_must_not_be_blank() {
        assert_eq!(title("  buy milk \n").as_str(), "buy milk");
        assert_eq!(ItemTitle::parse(" \t"), Err(DomainError::EmptyTitle));
    }

    #[test]
    fn title_longer_than_the_limit_is_rejected() {
        assert!(ItemTitle::parse(&"é".repeat(MAX_TITLE_CHARS)).is_ok());
        assert_eq!(
            ItemTitle::parse(&"é".repeat(MAX_TITLE_CHARS + 1)),
            Err(DomainError::TitleTooLong)
        );
    }

    #[test]
    fn ids_continue_after_the_highest_restored_id() {
        let mut list = ItemList::from_items(vec![item(4), item(1)]).unwrap();
        assert_eq!(list.add(title("next")), Ok(ItemId::new(5)));
    }

    #[test]
    fn restoring_duplicate_ids_fails() {
        assert_eq!(
            ItemList::from_items(vec![item(2), item(2)]),
            Err(RestoreError::DuplicateItem(ItemId::new(2)))
        );
    }

    #[test]
    fn restoring_the_largest_possible_id_fails() {
        assert_eq!(
            ItemList::from_items(vec![item(u64::MAX)]),
            Err(RestoreError::IdOutOfRange(ItemId::new(u64::MAX)))
        );
    }

    #[test]
    fn adding_after_the_last_possible_id_fails_without_changing_the_list() {
        let mut list = ItemList::from_items(vec![item(u64::MAX - 1)]).unwrap();
        assert_eq!(list.add(title("last")), Err(DomainError::NoIdsLeft));
        assert_eq!(list.add(title("again")), Err(DomainError::NoIdsLeft));
        assert_eq!(list.items().len(), 1);
    }

    #[test]
    fn toggling_an_unknown_item_fails() {
        let mut list = ItemList::default();
        assert_eq!(
            list.toggle(ItemId::new(9)),
            Err(DomainError::UnknownItem(ItemId::new(9)))
        );
    }

    #[test]
    fn removing_returns_the_item_and_keeps_the_order() {
        let mut list = ItemList::default();
        let first = list.add(title("first")).unwrap();
        let second = list.add(title("second")).unwrap();
        let third = list.add(title("third")).unwrap();
        let removed = list.remove(second).unwrap();
        assert_eq!(removed.title().as_str(), "second");
        let remaining: Vec<ItemId> = list.items().iter().map(Item::id).collect();
        assert_eq!(remaining, [first, third]);
    }
}
