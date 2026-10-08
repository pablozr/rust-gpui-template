//! Primitives shared by every view.
//!
//! Each takes the [`Theme`] so it never reaches for a raw value. Interactive primitives return
//! a `Stateful<Div>` so callers attach `on_click`, `aria_label` and `tooltip`.

use gpui::prelude::*;
use gpui::{Div, ElementId, Hsla, SharedString, Stateful, Svg, div, svg};

use crate::tokens::{self, radius, size, space, typography};
use crate::{Icon, Theme};

/// Visual weight of a [`button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    /// The one main action of a region (accent fill).
    Primary,
    /// Text only, until hovered.
    Ghost,
}

/// A text button.
pub fn button(
    theme: &Theme,
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    kind: ButtonKind,
) -> Stateful<Div> {
    let colors = theme.colors;
    let base = theme
        .text(div(), typography::BODY)
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .px(space::S4)
        .py(space::S2)
        .rounded(radius::M)
        .tab_index(0)
        .focus_visible(move |style| style.shadow(tokens::focus_ring(colors.focus_ring)))
        .cursor_pointer()
        .child(label.into());
    match kind {
        ButtonKind::Primary => base
            .bg(colors.accent)
            .text_color(colors.on_accent)
            .hover(move |style| style.bg(colors.accent_hover)),
        ButtonKind::Ghost => base
            .text_color(colors.text_muted)
            .hover(move |style| style.bg(colors.surface_hover).text_color(colors.text)),
    }
}

/// A square button that shows only an icon. Callers add `aria_label` and `tooltip`.
pub fn icon_button(theme: &Theme, id: impl Into<ElementId>, icon_kind: Icon) -> Stateful<Div> {
    let colors = theme.colors;
    div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(size::ICON_BUTTON)
        .rounded(radius::M)
        .tab_index(0)
        .focus_visible(move |style| style.shadow(tokens::focus_ring(colors.focus_ring)))
        .cursor_pointer()
        .hover(move |style| style.bg(colors.surface_hover))
        .child(icon(icon_kind, colors.text_muted))
}

/// A checkbox. Callers add `on_click` and `aria_label`.
pub fn checkbox(theme: &Theme, id: impl Into<ElementId>, checked: bool) -> Stateful<Div> {
    let colors = theme.colors;
    let base = div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(size::CHECKBOX)
        .rounded(radius::S)
        .border_1()
        .tab_index(0)
        .focus_visible(move |style| style.shadow(tokens::focus_ring(colors.focus_ring)))
        .cursor_pointer();
    if checked {
        base.bg(colors.accent)
            .border_color(colors.accent)
            .child(icon(Icon::Check, colors.on_accent))
    } else {
        base.border_color(colors.text_muted)
    }
}

/// A glyph of the icon set, tinted with `color`.
pub fn icon(icon: Icon, color: Hsla) -> Svg {
    svg().path(icon.path()).size(size::ICON).text_color(color)
}

/// What a region shows when it has no content yet: what it is for and how to fill it.
pub fn empty_state(
    theme: &Theme,
    title: impl Into<SharedString>,
    hint: impl Into<SharedString>,
) -> Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(space::S2)
        .child(
            theme
                .text(div(), typography::TITLE)
                .text_color(theme.colors.text)
                .child(title.into()),
        )
        .child(
            theme
                .text(div(), typography::BODY)
                .text_color(theme.colors.text_muted)
                .child(hint.into()),
        )
}

/// A recoverable error. Callers add the action button as a child.
pub fn error_banner(theme: &Theme, message: impl Into<SharedString>) -> Div {
    theme
        .text(div(), typography::BODY)
        .flex()
        .items_center()
        .justify_between()
        .gap(space::S3)
        .px(space::S4)
        .py(space::S2)
        .rounded(radius::L)
        .border_1()
        .border_color(theme.colors.danger)
        .bg(theme.colors.danger_soft)
        .text_color(theme.colors.text)
        .child(message.into())
}
