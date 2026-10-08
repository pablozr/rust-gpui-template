//! The theme: the one object views use to reach tokens.

use gpui::{App, Global, Styled};

use crate::tokens::{self, Palette, TypeToken};

/// Dark is the default.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ThemeMode {
    /// Light text on dark surfaces.
    #[default]
    Dark,
    /// Dark text on light surfaces.
    Light,
}

impl Global for ThemeMode {}

impl ThemeMode {
    /// The other mode.
    pub fn toggled(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }
}

/// Resolved tokens for one mode. Build it once per render with [`Theme::of`].
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    /// The mode the colors belong to.
    pub mode: ThemeMode,
    /// The colors of that mode.
    pub colors: Palette,
}

impl Theme {
    /// The tokens of `mode`.
    pub fn new(mode: ThemeMode) -> Self {
        let colors = match mode {
            ThemeMode::Dark => Palette::dark(),
            ThemeMode::Light => Palette::light(),
        };
        Self { mode, colors }
    }

    /// The theme for the app's current mode.
    pub fn of(cx: &App) -> Self {
        Self::new(cx.try_global::<ThemeMode>().copied().unwrap_or_default())
    }

    /// Applies a step of the type scale (family, size, line height, weight).
    pub fn text<E: Styled>(&self, element: E, token: TypeToken) -> E {
        element
            .font_family(tokens::FONT_FAMILY)
            .text_size(token.size)
            .line_height(token.line_height)
            .font_weight(token.weight)
    }
}

/// Switches between dark and light and redraws every window.
pub fn toggle(cx: &mut App) {
    let next = Theme::of(cx).mode.toggled();
    cx.set_global(next);
    cx.refresh_windows();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_and_light_palettes_differ() {
        let dark = Theme::new(ThemeMode::Dark).colors;
        let light = Theme::new(ThemeMode::Light).colors;
        assert_ne!(dark.background, light.background);
        assert_ne!(dark.text, light.text);
    }

    #[test]
    fn toggling_twice_returns_the_same_mode() {
        assert_eq!(ThemeMode::Dark.toggled().toggled(), ThemeMode::Dark);
        assert_eq!(ThemeMode::Light.toggled(), ThemeMode::Dark);
    }
}
