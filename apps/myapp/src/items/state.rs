//! State of the items view as plain data: no GPUI, so it is tested without a window.

use myapp_core::Problem;
use myapp_core::domain::{Item, ItemId, ItemList};
use myapp_core::items::{self, Command};

/// What the view shows.
#[derive(Debug, Default, PartialEq, Eq)]
pub enum Phase {
    /// The saved items are being read.
    #[default]
    Loading,
    /// The list is usable, possibly empty.
    Ready,
    /// The saved items could not be read.
    LoadFailed(Problem),
}

/// What the view must do after a retry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryEffect {
    /// Read the saved items again.
    Load,
    /// Start a save now.
    Save,
    /// A save is already running and will pick up the latest list.
    Nothing,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Saving {
    #[default]
    Idle,
    InFlight,
    /// The list changed while a save was running, so another save must follow.
    InFlightDirty,
}

/// The list, the selection and the bookkeeping around loading and saving.
#[derive(Debug, Default)]
pub struct ItemsState {
    phase: Phase,
    list: ItemList,
    selected: Option<ItemId>,
    notice: Option<Problem>,
    saving: Saving,
}

impl ItemsState {
    /// Goes back to the loading phase.
    pub fn start_loading(&mut self) {
        self.phase = Phase::Loading;
    }

    /// Takes the result of a load: the list on success, the problem otherwise.
    pub fn finish_loading(&mut self, result: myapp_core::Result<ItemList>) {
        match result {
            Ok(list) => {
                self.selected = list.items().first().map(Item::id);
                self.list = list;
                self.notice = None;
                self.phase = Phase::Ready;
            }
            Err(problem) => self.phase = Phase::LoadFailed(problem),
        }
    }

    /// Applies a command. Returns whether the list changed; a rejected command leaves a notice.
    pub fn apply(&mut self, command: Command) -> bool {
        let removed_at = match command {
            Command::Remove(id) => self.list.position(id),
            Command::Add { .. } | Command::Toggle(_) => None,
        };
        match items::apply(&mut self.list, command) {
            Ok(id) => {
                self.notice = None;
                self.selected = match removed_at {
                    Some(index) if self.selected == Some(id) => self.neighbor_at(index),
                    Some(_) => self.selected,
                    None => Some(id),
                };
                true
            }
            Err(problem) => {
                self.notice = Some(problem);
                false
            }
        }
    }

    /// Returns whether a save must start now. During a save it only marks the list as dirty.
    pub fn request_save(&mut self) -> bool {
        match self.saving {
            Saving::Idle => {
                self.saving = Saving::InFlight;
                true
            }
            Saving::InFlight | Saving::InFlightDirty => {
                self.saving = Saving::InFlightDirty;
                false
            }
        }
    }

    /// Takes the result of a save. Returns whether another save must start now.
    pub fn finish_saving(&mut self, result: myapp_core::Result<()>) -> bool {
        if let Err(problem) = result {
            self.notice = Some(problem);
        }
        let again = self.saving == Saving::InFlightDirty;
        self.saving = if again {
            Saving::InFlight
        } else {
            Saving::Idle
        };
        again
    }

    /// Reacts to a retry request: reload after a failed load, otherwise drop the notice and save.
    pub fn retry(&mut self) -> RetryEffect {
        if matches!(self.phase, Phase::LoadFailed(_)) {
            return RetryEffect::Load;
        }
        self.notice = None;
        if self.request_save() {
            RetryEffect::Save
        } else {
            RetryEffect::Nothing
        }
    }

    /// Selects the item below the selected one, or the first when nothing is selected.
    pub fn select_next(&mut self) {
        let index = self.selected_index().map_or(0, |index| index + 1);
        self.select_at(index);
    }

    /// Selects the item above the selected one, or the first when nothing is selected.
    pub fn select_previous(&mut self) {
        let index = self
            .selected_index()
            .map_or(0, |index| index.saturating_sub(1));
        self.select_at(index);
    }

    /// The selected item.
    pub fn selected(&self) -> Option<ItemId> {
        self.selected
    }

    /// Where the selected item sits in the list.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected.and_then(|id| self.list.position(id))
    }

    /// The list, e.g. to render it or snapshot it for a save.
    pub fn list(&self) -> &ItemList {
        &self.list
    }

    /// What the view shows.
    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    /// The problem to show next to the list, if any.
    pub fn notice(&self) -> Option<&Problem> {
        self.notice.as_ref()
    }

    fn select_at(&mut self, index: usize) {
        if let Some(item) = self.list.items().get(index) {
            self.selected = Some(item.id());
        }
    }

    /// The item now at `index`, or the last one when `index` is past the end.
    fn neighbor_at(&self, index: usize) -> Option<ItemId> {
        let items = self.list.items();
        items.get(index).or(items.last()).map(Item::id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn add(state: &mut ItemsState, title: &str) -> ItemId {
        assert!(state.apply(Command::Add {
            title: title.into()
        }));
        state.selected().unwrap()
    }

    fn ready_with(titles: &[&str]) -> (ItemsState, Vec<ItemId>) {
        let mut state = ItemsState::default();
        state.finish_loading(Ok(ItemList::default()));
        let ids = titles.iter().map(|title| add(&mut state, title)).collect();
        (state, ids)
    }

    #[test]
    fn failed_load_shows_the_problem() {
        let mut state = ItemsState::default();
        state.finish_loading(Err(Problem::StoreUnavailable));
        assert_eq!(state.phase(), &Phase::LoadFailed(Problem::StoreUnavailable));
    }

    #[test]
    fn adding_selects_the_new_item() {
        let (state, ids) = ready_with(&["one", "two"]);
        assert_eq!(state.selected(), Some(ids[1]));
        assert_eq!(state.selected_index(), Some(1));
    }

    #[test]
    fn removing_the_selected_item_selects_its_neighbor() {
        let (mut state, ids) = ready_with(&["one", "two", "three"]);
        state.select_previous();
        assert_eq!(state.selected(), Some(ids[1]));
        assert!(state.apply(Command::Remove(ids[1])));
        assert_eq!(state.selected(), Some(ids[2]));
        assert!(state.apply(Command::Remove(ids[2])));
        assert_eq!(state.selected(), Some(ids[0]));
    }

    #[test]
    fn selection_stops_at_the_ends() {
        let (mut state, ids) = ready_with(&["one", "two"]);
        state.select_next();
        assert_eq!(state.selected(), Some(ids[1]));
        state.select_previous();
        state.select_previous();
        assert_eq!(state.selected(), Some(ids[0]));
    }

    #[test]
    fn a_save_requested_while_saving_runs_once_more_with_the_latest_list() {
        let mut state = ItemsState::default();
        assert!(state.request_save());
        assert!(!state.request_save());
        assert!(state.finish_saving(Ok(())));
        assert!(!state.finish_saving(Ok(())));
        assert!(state.request_save());
    }

    #[test]
    fn failed_save_keeps_the_items_and_reports_the_problem() {
        let (mut state, _) = ready_with(&["one"]);
        assert!(state.request_save());
        assert!(!state.finish_saving(Err(Problem::StoreUnavailable)));
        assert_eq!(state.list().items().len(), 1);
        assert_eq!(state.notice(), Some(&Problem::StoreUnavailable));
    }

    #[test]
    fn retry_reloads_after_a_failed_load_and_saves_after_a_failed_save() {
        let mut state = ItemsState::default();
        state.finish_loading(Err(Problem::StoreUnavailable));
        assert_eq!(state.retry(), RetryEffect::Load);

        let (mut state, _) = ready_with(&["one"]);
        assert!(state.request_save());
        assert!(!state.finish_saving(Err(Problem::StoreUnavailable)));
        assert_eq!(state.retry(), RetryEffect::Save);
        assert_eq!(state.notice(), None);
        assert_eq!(state.retry(), RetryEffect::Nothing);
    }
}
