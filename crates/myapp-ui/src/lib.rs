//! The design system. Three rules:
//! raw values (colors, sizes, spacing) live only in `tokens`;
//! views reach tokens through `Theme::of(cx)`;
//! nothing here knows a business type.

pub mod assets;
pub mod components;
pub mod theme;
pub mod tokens;
pub mod tooltip;

pub use assets::{Assets, Icon};
pub use theme::{Theme, ThemeMode};
