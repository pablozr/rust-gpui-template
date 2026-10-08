//! Design tokens: the single source of raw values, in both themes.

use gpui::{BoxShadow, FontWeight, Hsla, Pixels, Rgba, point, px, rgb};

/// `0xRRGGBB` at `alpha`. Keeps hex and alpha apart so a six-digit color is
/// never read as eight digits.
fn tone(hex: u32, alpha: f32) -> Hsla {
    let mut color: Rgba = rgb(hex);
    color.a = alpha;
    color.into()
}

/// Colors that change between the dark and the light theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    /// Window background.
    pub background: Hsla,
    /// Raised areas: rows, tooltips.
    pub surface: Hsla,
    /// A surface under the pointer or selected.
    pub surface_hover: Hsla,
    /// Dividers and outlines.
    pub border: Hsla,
    /// Primary text.
    pub text: Hsla,
    /// Secondary text and idle icons.
    pub text_muted: Hsla,
    /// Primary action fill.
    pub accent: Hsla,
    /// Primary action fill under the pointer.
    pub accent_hover: Hsla,
    /// Text and icons on an accent fill.
    pub on_accent: Hsla,
    /// Error outline.
    pub danger: Hsla,
    /// Error fill.
    pub danger_soft: Hsla,
    /// Keyboard focus ring.
    pub focus_ring: Hsla,
    /// No color at all.
    pub transparent: Hsla,
}

impl Palette {
    /// The dark theme.
    pub fn dark() -> Self {
        Self {
            background: tone(0x131316, 1.0),
            surface: tone(0x1C1C21, 1.0),
            surface_hover: tone(0x27272E, 1.0),
            border: tone(0xFFFFFF, 0.12),
            text: tone(0xF2F2F5, 1.0),
            text_muted: tone(0xA1A1AA, 1.0),
            accent: tone(0x6366F1, 1.0),
            accent_hover: tone(0x7C7FF5, 1.0),
            on_accent: tone(0xFFFFFF, 1.0),
            danger: tone(0xF87171, 1.0),
            danger_soft: tone(0xF87171, 0.14),
            focus_ring: tone(0xA5B4FC, 1.0),
            transparent: tone(0x000000, 0.0),
        }
    }

    /// The light theme.
    pub fn light() -> Self {
        Self {
            background: tone(0xF7F7F9, 1.0),
            surface: tone(0xFFFFFF, 1.0),
            surface_hover: tone(0xECECF1, 1.0),
            border: tone(0x000000, 0.12),
            text: tone(0x18181B, 1.0),
            text_muted: tone(0x52525B, 1.0),
            accent: tone(0x4F46E5, 1.0),
            accent_hover: tone(0x4338CA, 1.0),
            on_accent: tone(0xFFFFFF, 1.0),
            danger: tone(0xB91C1C, 1.0),
            danger_soft: tone(0xB91C1C, 0.10),
            focus_ring: tone(0x4F46E5, 1.0),
            transparent: tone(0x000000, 0.0),
        }
    }
}

/// The font family of all text: the platform's interface font.
pub const FONT_FAMILY: &str = ".SystemUIFont";

/// The 4 px spacing scale.
pub mod space {
    use super::*;
    /// Hairline gaps.
    pub const S1: Pixels = px(4.0);
    /// Inside compact controls.
    pub const S2: Pixels = px(8.0);
    /// Between related elements.
    pub const S3: Pixels = px(12.0);
    /// Page and panel padding.
    pub const S4: Pixels = px(16.0);
    /// Between sections.
    pub const S5: Pixels = px(24.0);
    /// Around large empty areas.
    pub const S6: Pixels = px(32.0);
}

/// Corner radii.
pub mod radius {
    use super::*;
    /// Checkboxes.
    pub const S: Pixels = px(4.0);
    /// Buttons and rows.
    pub const M: Pixels = px(8.0);
    /// Banners and tooltips.
    pub const L: Pixels = px(12.0);
}

/// One step of the type scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeToken {
    /// Font size.
    pub size: Pixels,
    /// Distance between baselines.
    pub line_height: Pixels,
    /// Boldness.
    pub weight: FontWeight,
}

const fn type_token(size: f32, line_height: f32, weight: f32) -> TypeToken {
    TypeToken {
        size: px(size),
        line_height: px(line_height),
        weight: FontWeight(weight),
    }
}

/// The type scale.
pub mod typography {
    use super::*;
    /// Window and section titles.
    pub const TITLE: TypeToken = type_token(18.0, 24.0, 700.0);
    /// Running text, labels and buttons.
    pub const BODY: TypeToken = type_token(14.0, 20.0, 500.0);
    /// Tooltips and fine print.
    pub const CAPTION: TypeToken = type_token(12.0, 16.0, 400.0);
}

/// Fixed dimensions of components.
pub mod size {
    use super::*;
    /// One item row; the list measures it once.
    pub const ROW_HEIGHT: Pixels = px(44.0);
    /// Lucide icons (24 px grid, 2 px stroke).
    pub const ICON: Pixels = px(16.0);
    /// Square icon-only buttons.
    pub const ICON_BUTTON: Pixels = px(32.0);
    /// Side of a checkbox.
    pub const CHECKBOX: Pixels = px(20.0);
    /// The bar at the top of the window.
    pub const HEADER_HEIGHT: Pixels = px(52.0);
}

/// The main window.
pub mod window {
    use gpui::{Pixels, Size, px, size};

    /// Size the window opens with.
    pub const DEFAULT_SIZE: Size<Pixels> = size(px(640.0), px(560.0));
    /// Smallest size the window can be resized to.
    pub const MIN_SIZE: Size<Pixels> = size(px(420.0), px(360.0));
}

/// The keyboard focus ring: a solid outline that does not shift the layout.
pub fn focus_ring(color: Hsla) -> Vec<BoxShadow> {
    vec![BoxShadow {
        color,
        offset: point(px(0.0), px(0.0)),
        blur_radius: px(0.0),
        spread_radius: px(2.0),
        inset: false,
    }]
}
