//! The window's frame: header, theme switch and global shortcuts.

use std::sync::Arc;

use gpui::prelude::*;
use gpui::{App, Context, Entity, Focusable, KeyBinding, Window, actions, div};
use myapp_core::ItemStore;
use myapp_ui::Theme;
use myapp_ui::components::{ButtonKind, button};
use myapp_ui::tokens::{size, space, typography};

use crate::i18n;
use crate::items::ItemsView;

actions!(shell, [ToggleTheme, Quit]);

const CONTEXT: &str = "Shell";

/// Registers the global shortcuts and the quit action.
pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("secondary-shift-t", ToggleTheme, Some(CONTEXT)),
        KeyBinding::new("secondary-q", Quit, None),
    ]);
    cx.on_action(|_: &Quit, cx| cx.quit());
}

/// The root view of the main window.
pub struct Shell {
    items: Entity<ItemsView>,
}

impl Shell {
    /// Creates the items view and gives it the keyboard focus.
    pub fn new(store: Arc<dyn ItemStore>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let items = cx.new(|cx| ItemsView::new(store, cx));
        let handle = items.focus_handle(cx);
        window.focus(&handle, cx);
        Self { items }
    }
}

impl Render for Shell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(cx);
        let header = div()
            .flex()
            .flex_none()
            .items_center()
            .justify_between()
            .h(size::HEADER_HEIGHT)
            .px(space::S4)
            .border_b_1()
            .border_color(theme.colors.border)
            .child(
                theme
                    .text(div(), typography::TITLE)
                    .child(i18n::app::title()),
            )
            .child(
                button(
                    &theme,
                    "toggle-theme",
                    i18n::app::toggle_theme(),
                    ButtonKind::Ghost,
                )
                .on_click(cx.listener(|_, _, _window, cx| myapp_ui::theme::toggle(cx))),
            );
        div()
            .key_context(CONTEXT)
            .on_action(cx.listener(|_, _: &ToggleTheme, _window, cx| myapp_ui::theme::toggle(cx)))
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .child(header)
            .child(div().flex_1().min_h_0().child(self.items.clone()))
    }
}
