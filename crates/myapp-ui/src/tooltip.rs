//! A text tooltip for icon-only controls.

use gpui::prelude::*;
use gpui::{AnyView, App, Context, SharedString, Window, div};

use crate::Theme;
use crate::tokens::{radius, space, typography};

/// The tooltip view.
#[derive(Debug)]
pub struct Tooltip {
    text: SharedString,
}

impl Render for Tooltip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(cx);
        theme
            .text(div(), typography::CAPTION)
            .px(space::S2)
            .py(space::S1)
            .rounded(radius::M)
            .border_1()
            .border_color(theme.colors.border)
            .bg(theme.colors.surface)
            .text_color(theme.colors.text)
            .child(self.text.clone())
    }
}

/// A tooltip builder for `.tooltip(..)` that shows `text`.
pub fn text(text: impl Into<SharedString>) -> impl Fn(&mut Window, &mut App) -> AnyView {
    let text = text.into();
    move |_window, cx| cx.new(|_| Tooltip { text: text.clone() }).into()
}
