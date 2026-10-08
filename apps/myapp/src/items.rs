//! The items feature: a list you can add to, tick off and remove, kept on disk.

mod state;

use std::ops::Range;
use std::sync::Arc;

use gpui::prelude::*;
use gpui::{
    App, Context, Div, FocusHandle, Focusable, KeyBinding, SharedString, Stateful, Task,
    UniformListScrollHandle, Window, actions, div, uniform_list,
};
use myapp_core::ItemStore;
use myapp_core::Problem;
use myapp_core::domain::Item;
use myapp_core::items::{self as core_items, Command};
use myapp_ui::Theme;
use myapp_ui::components::{
    BUTTON_CONTEXT, ButtonKind, button, checkbox, empty_state, error_banner, icon_button,
};
use myapp_ui::tokens::{radius, size, space, typography};
use myapp_ui::{Icon, tooltip};
use state::{ItemsState, Phase, RetryEffect};

use crate::i18n;

actions!(
    items,
    [
        AddItem,
        ToggleSelected,
        RemoveSelected,
        SelectNext,
        SelectPrevious,
        Retry
    ]
);

const CONTEXT: &str = "Items";

/// Registers the keyboard shortcuts of the items view.
pub(crate) fn bind_keys(cx: &mut App) {
    // A focused button activates on Space itself, so the list keys that clash step aside.
    let list = format!("{CONTEXT} && !{BUTTON_CONTEXT}");
    cx.bind_keys([
        KeyBinding::new("secondary-n", AddItem, Some(CONTEXT)),
        KeyBinding::new("secondary-r", Retry, Some(CONTEXT)),
        KeyBinding::new("space", ToggleSelected, Some(&list)),
        KeyBinding::new("delete", RemoveSelected, Some(&list)),
        KeyBinding::new("backspace", RemoveSelected, Some(&list)),
        KeyBinding::new("down", SelectNext, Some(CONTEXT)),
        KeyBinding::new("up", SelectPrevious, Some(CONTEXT)),
    ]);
}

/// Turns user intent into core calls and renders the resulting state.
pub(crate) struct ItemsView {
    state: ItemsState,
    store: Arc<dyn ItemStore>,
    focus_handle: FocusHandle,
    scroll: UniformListScrollHandle,
    load_task: Option<Task<()>>,
    save_task: Option<Task<()>>,
}

impl ItemsView {
    /// Creates the view and starts loading the saved items.
    pub(crate) fn new(store: Arc<dyn ItemStore>, cx: &mut Context<Self>) -> Self {
        let mut view = Self {
            state: ItemsState::default(),
            store,
            focus_handle: cx.focus_handle().tab_stop(true),
            scroll: UniformListScrollHandle::new(),
            load_task: None,
            save_task: None,
        };
        view.load(cx);
        Self::flush_saves_on_quit(cx);
        view
    }

    /// Makes quitting write the changes that are not on disk yet.
    fn flush_saves_on_quit(cx: &mut Context<Self>) {
        cx.on_app_quit(|view, cx| {
            // Awaiting the running save would wait on the main thread, which quitting blocks.
            // Dropping it cancels it unless it is mid-write; the store serializes the writes.
            view.save_task = None;
            let unsaved = view
                .state
                .has_unsaved_changes()
                .then(|| (Arc::clone(&view.store), view.state.list().clone()));
            let executor = cx.background_executor().clone();
            async move {
                let Some((store, snapshot)) = unsaved else {
                    return;
                };
                let saved = executor
                    .spawn(async move { core_items::save(store.as_ref(), &snapshot) })
                    .await;
                if let Err(problem) = saved {
                    tracing::error!(%problem, "the last changes could not be saved on quit");
                }
            }
        })
        .detach();
    }

    fn load(&mut self, cx: &mut Context<Self>) {
        self.state.start_loading();
        let store = Arc::clone(&self.store);
        let loading = cx.background_spawn(async move { core_items::load(store.as_ref()) });
        // Replacing the task cancels a load that is still running.
        self.load_task = Some(cx.spawn(async move |this, cx| {
            let result = loading.await;
            let updated = this.update(cx, |view, cx| {
                view.state.finish_loading(result);
                cx.notify();
            });
            if let Err(error) = updated {
                tracing::debug!(%error, "items view closed before loading finished");
            }
        }));
        cx.notify();
    }

    fn save(&mut self, cx: &mut Context<Self>) {
        let store = Arc::clone(&self.store);
        let snapshot = self.state.list().clone();
        let saving =
            cx.background_spawn(async move { core_items::save(store.as_ref(), &snapshot) });
        self.save_task = Some(cx.spawn(async move |this, cx| {
            let result = saving.await;
            let updated = this.update(cx, |view, cx| {
                if view.state.finish_saving(result) {
                    view.save(cx);
                }
                cx.notify();
            });
            if let Err(error) = updated {
                tracing::debug!(%error, "items view closed before saving finished");
            }
        }));
    }

    fn run(&mut self, command: Command, cx: &mut Context<Self>) {
        if self.state.apply(command) && self.state.request_save() {
            self.save(cx);
        }
        cx.notify();
    }

    fn reveal_selection(&self) {
        if let Some(index) = self.state.selected_index() {
            self.scroll
                .scroll_to_item(index, gpui::ScrollStrategy::Nearest);
        }
    }

    fn add_item(&mut self, _: &AddItem, _window: &mut Window, cx: &mut Context<Self>) {
        let title = i18n::items::untitled_item(self.state.list().items().len() + 1);
        self.run(Command::Add { title }, cx);
        self.reveal_selection();
    }

    fn toggle_selected(
        &mut self,
        _: &ToggleSelected,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(id) = self.state.selected() {
            self.run(Command::Toggle(id), cx);
        }
    }

    fn remove_selected(
        &mut self,
        _: &RemoveSelected,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(id) = self.state.selected() {
            self.run(Command::Remove(id), cx);
            self.reveal_selection();
        }
    }

    fn select_next(&mut self, _: &SelectNext, _window: &mut Window, cx: &mut Context<Self>) {
        self.state.select_next();
        self.reveal_selection();
        cx.notify();
    }

    fn select_previous(
        &mut self,
        _: &SelectPrevious,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.state.select_previous();
        self.reveal_selection();
        cx.notify();
    }

    fn retry(&mut self, _: &Retry, _window: &mut Window, cx: &mut Context<Self>) {
        match self.state.retry() {
            RetryEffect::Load => self.load(cx),
            RetryEffect::Save => self.save(cx),
            RetryEffect::Nothing => {}
        }
        cx.notify();
    }

    fn add_button(&self, theme: &Theme, cx: &Context<Self>) -> Stateful<Div> {
        button(
            theme,
            "add-item",
            i18n::items::add_item(),
            ButtonKind::Primary,
        )
        .on_click(cx.listener(|this, _, window, cx| this.add_item(&AddItem, window, cx)))
    }

    fn banner(
        &self,
        theme: &Theme,
        problem: &Problem,
        id: &'static str,
        cx: &Context<Self>,
    ) -> Div {
        let banner = error_banner(theme, i18n::problems::describe(problem));
        if !problem.is_retryable() {
            return banner;
        }
        banner.child(
            button(theme, id, i18n::items::retry(), ButtonKind::Ghost)
                .on_click(cx.listener(|this, _, window, cx| this.retry(&Retry, window, cx))),
        )
    }

    fn content(&self, theme: &Theme, cx: &Context<Self>) -> gpui::AnyElement {
        match self.state.phase() {
            Phase::Loading => theme
                .text(div(), typography::BODY)
                .text_color(theme.colors.text_muted)
                .child(i18n::items::loading())
                .into_any_element(),
            Phase::LoadFailed(problem) => self
                .banner(theme, problem, "retry-load", cx)
                .into_any_element(),
            Phase::Ready if self.state.list().items().is_empty() => div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .child(
                    empty_state(theme, i18n::items::empty_title(), i18n::items::empty_hint())
                        .child(
                            button(
                                theme,
                                "empty-add",
                                i18n::items::add_item(),
                                ButtonKind::Primary,
                            )
                            .on_click(cx.listener(
                                |this, _, window, cx| this.add_item(&AddItem, window, cx),
                            )),
                        ),
                )
                .into_any_element(),
            Phase::Ready => uniform_list(
                "item-rows",
                self.state.list().items().len(),
                cx.processor(|this, range: Range<usize>, _window, cx| {
                    let theme = Theme::of(cx);
                    let selected = this.state.selected();
                    this.state
                        .list()
                        .items()
                        .get(range)
                        .unwrap_or_default()
                        .iter()
                        .map(|item| row(&theme, item, selected == Some(item.id()), cx))
                        .collect()
                }),
            )
            .track_scroll(&self.scroll)
            .flex_1()
            .min_h_0()
            .into_any_element(),
        }
    }
}

fn row(theme: &Theme, item: &Item, selected: bool, cx: &Context<ItemsView>) -> Stateful<Div> {
    let id = item.id();
    let toggle_label = if item.is_done() {
        i18n::items::mark_not_done()
    } else {
        i18n::items::mark_done()
    };
    let title_color = if item.is_done() {
        theme.colors.text_muted
    } else {
        theme.colors.text
    };
    div()
        .id(("item", id.get()))
        .flex()
        .items_center()
        .gap(space::S3)
        .px(space::S3)
        .h(size::ROW_HEIGHT)
        .rounded(radius::M)
        .when(selected, |row| row.bg(theme.colors.surface_hover))
        .child(
            checkbox(theme, ("toggle", id.get()), item.is_done())
                .aria_label(toggle_label)
                .on_click(cx.listener(move |this, _, _window, cx| {
                    this.run(Command::Toggle(id), cx);
                })),
        )
        .child(
            theme
                .text(div(), typography::BODY)
                .flex_1()
                .truncate()
                .text_color(title_color)
                .child(SharedString::from(item.title().to_shared())),
        )
        .child(
            icon_button(theme, ("remove", id.get()), Icon::X)
                .aria_label(i18n::items::remove_item())
                .tooltip(tooltip::text(i18n::items::remove_item()))
                .on_click(cx.listener(move |this, _, _window, cx| {
                    this.run(Command::Remove(id), cx);
                })),
        )
}

impl Focusable for ItemsView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ItemsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(cx);
        let mut root = div()
            .id("items")
            .key_context(CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::add_item))
            .on_action(cx.listener(Self::toggle_selected))
            .on_action(cx.listener(Self::remove_selected))
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_previous))
            .on_action(cx.listener(Self::retry))
            .flex()
            .flex_col()
            .gap(space::S3)
            .p(space::S4)
            .size_full();
        if matches!(self.state.phase(), Phase::Ready) {
            root = root.child(self.add_button(&theme, cx));
        }
        if let Some(problem) = self.state.notice() {
            root = root.child(self.banner(&theme, problem, "retry-save", cx));
        }
        root.child(self.content(&theme, cx))
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Entity, InputEvent as _, KeyUpEvent, Keystroke, TestAppContext, VisualTestContext};
    use myapp_core::domain::{ItemId, ItemTitle};
    use myapp_core::fake::{Failure, MemoryStore};

    use super::*;

    fn open<'a>(
        store: &Arc<MemoryStore>,
        cx: &'a mut TestAppContext,
    ) -> (Entity<ItemsView>, &'a mut VisualTestContext) {
        let port: Arc<dyn ItemStore> = Arc::<MemoryStore>::clone(store);
        let (view, cx) = cx.add_window_view(|_window, cx| ItemsView::new(port, cx));
        cx.update(|_window, cx| bind_keys(cx));
        let handle = view.read_with(cx, |view, _| view.focus_handle.clone());
        cx.update(|window, cx| window.focus(&handle, cx));
        cx.run_until_parked();
        (view, cx)
    }

    #[gpui::test]
    fn secondary_n_adds_an_item_and_saves_it(cx: &mut TestAppContext) {
        let store = Arc::new(MemoryStore::default());
        let (view, cx) = open(&store, cx);

        cx.simulate_keystrokes("secondary-n");
        cx.run_until_parked();

        assert_eq!(
            view.read_with(cx, |view, _| view.state.list().items().len()),
            1
        );
        assert_eq!(store.saved().len(), 1);
    }

    #[gpui::test]
    fn failed_save_keeps_the_item_and_reports_the_problem(cx: &mut TestAppContext) {
        let store = Arc::new(MemoryStore::default());
        let (view, cx) = open(&store, cx);
        store.fail_with(Some(Failure::Unavailable));

        cx.simulate_keystrokes("secondary-n");
        cx.run_until_parked();

        assert_eq!(
            view.read_with(cx, |view, _| view.state.list().items().len()),
            1
        );
        let notice = view.read_with(cx, |view, _| view.state.notice().cloned());
        assert_eq!(notice, Some(Problem::StoreUnavailable));
    }

    fn saved_item(id: u64, title: &str) -> Item {
        Item::new(ItemId::new(id), ItemTitle::parse(title).unwrap(), false)
    }

    fn done_flags(view: &Entity<ItemsView>, cx: &mut VisualTestContext) -> Vec<bool> {
        view.read_with(cx, |view, _| {
            view.state
                .list()
                .items()
                .iter()
                .map(Item::is_done)
                .collect()
        })
    }

    #[gpui::test]
    fn space_on_a_focused_checkbox_toggles_its_row_not_the_selected_one(cx: &mut TestAppContext) {
        let store = Arc::new(MemoryStore::with_items(vec![
            saved_item(1, "one"),
            saved_item(2, "two"),
        ]));
        let (view, cx) = open(&store, cx);
        cx.simulate_keystrokes("down");
        assert_eq!(done_flags(&view, cx), [false, false]);

        // Tab stops in order: the add button, then the first row's checkbox.
        cx.update(|window, cx| {
            window.focus_next(cx);
            window.focus_next(cx);
        });
        let on_button = cx.update(|window, _| {
            window
                .context_stack()
                .last()
                .is_some_and(|context| context.contains(BUTTON_CONTEXT))
        });
        assert!(on_button, "a button should hold the focus");
        // A focused element only handles keys after it has been painted as focused.
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.simulate_keystrokes("space");
        cx.update(|window, cx| {
            let release = KeyUpEvent {
                keystroke: Keystroke::parse("space").unwrap(),
            };
            window.dispatch_event(release.to_platform_input(), cx);
        });

        assert_eq!(done_flags(&view, cx), [true, false]);
    }

    #[gpui::test]
    fn space_on_the_list_toggles_the_selected_item(cx: &mut TestAppContext) {
        let store = Arc::new(MemoryStore::with_items(vec![saved_item(1, "one")]));
        let (view, cx) = open(&store, cx);

        cx.simulate_keystrokes("space");

        assert_eq!(done_flags(&view, cx), [true]);
    }

    #[gpui::test]
    fn commands_before_the_load_finishes_are_ignored(cx: &mut TestAppContext) {
        let store = Arc::new(MemoryStore::with_items(vec![saved_item(1, "one")]));
        let port: Arc<dyn ItemStore> = Arc::<MemoryStore>::clone(&store);
        let (view, cx) = cx.add_window_view(|_window, cx| {
            let mut view = ItemsView::new(port, cx);
            view.run(
                Command::Add {
                    title: "new".into(),
                },
                cx,
            );
            view
        });
        cx.run_until_parked();

        assert_eq!(done_flags(&view, cx).len(), 1);
        assert_eq!(store.saved().len(), 1);
    }

    #[gpui::test]
    fn quitting_saves_what_changed_during_the_running_save(cx: &mut TestAppContext) {
        let store = Arc::new(MemoryStore::default());
        let (view, cx) = open(&store, cx);

        view.update(cx, |view, cx| {
            for title in ["one", "two"] {
                view.run(
                    Command::Add {
                        title: title.into(),
                    },
                    cx,
                );
            }
        });
        cx.cx.update(|cx| cx.shutdown());

        assert_eq!(store.saved().len(), 2);
    }
}
