//! Embedded images: the Lucide icons (ISC, see `assets/icons/LICENSE-lucide.txt`).
//!
//! Compiled into the binary. Register [`Assets`] on the `Application` (`with_assets`) so
//! `svg().path(..)` finds them.

use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

macro_rules! embedded {
    ($($path:literal),+ $(,)?) => {
        &[$(($path, include_bytes!(concat!("../../../assets/", $path)))),+]
    };
}

const FILES: &[(&str, &[u8])] = embedded!["icons/check.svg", "icons/x.svg",];

/// Serves the embedded files by their path under `assets/`.
#[derive(Debug)]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(FILES
            .iter()
            .find(|(name, _)| *name == path)
            .map(|(_, bytes)| Cow::Borrowed(*bytes)))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(FILES
            .iter()
            .filter(|(name, _)| name.starts_with(path))
            .map(|(name, _)| SharedString::new_static(name))
            .collect())
    }
}

/// The icons views can draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    /// A cross.
    X,
    /// A check mark.
    Check,
}

impl Icon {
    /// Where the icon is served, for `svg().path(..)`.
    pub fn path(self) -> &'static str {
        match self {
            Self::X => "icons/x.svg",
            Self::Check => "icons/check.svg",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_embedded_file_is_served_and_unknown_paths_are_not() {
        for (name, bytes) in FILES {
            let loaded = Assets.load(name).unwrap().unwrap();
            assert_eq!(loaded.as_ref(), *bytes);
            assert!(bytes.starts_with(b"<svg"), "{name} is an SVG");
        }
        for icon in [Icon::X, Icon::Check] {
            assert!(Assets.load(icon.path()).unwrap().is_some(), "{icon:?}");
        }
        assert!(Assets.load("icons/missing.svg").unwrap().is_none());
    }
}
