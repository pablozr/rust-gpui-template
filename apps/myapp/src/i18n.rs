//! Interface text. Every string a person reads comes from here, never a literal in a view.
//!
//! `strings!` and `formats!` require a translation for every [`Language`], so adding a language
//! makes the compiler list every missing string.

use std::sync::atomic::{AtomicU8, Ordering};

/// Interface languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    PortugueseBrazil,
}

impl Language {
    const ALL: [Self; 2] = [Self::English, Self::PortugueseBrazil];

    /// The language for a tag such as `en` or `pt-BR`.
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "en" => Some(Self::English),
            "pt-BR" | "pt-br" | "pt" => Some(Self::PortugueseBrazil),
            _ => None,
        }
    }
}

/// Index into `Language::ALL` of the language in use.
static CURRENT: AtomicU8 = AtomicU8::new(0);

/// Sets the language in use.
pub fn set(language: Language) {
    let index = Language::ALL
        .iter()
        .position(|candidate| *candidate == language)
        .unwrap_or_default();
    CURRENT.store(index as u8, Ordering::Relaxed);
}

/// The language in use.
pub fn current() -> Language {
    Language::ALL
        .get(usize::from(CURRENT.load(Ordering::Relaxed)))
        .copied()
        .unwrap_or(Language::English)
}

macro_rules! strings {
    ($( $name:ident { en: $en:literal, pt_br: $pt_br:literal $(,)? } )*) => {
        $(
            pub fn $name() -> &'static str {
                match $crate::i18n::current() {
                    $crate::i18n::Language::English => $en,
                    $crate::i18n::Language::PortugueseBrazil => $pt_br,
                }
            }
        )*
    };
}

/// Like `strings!`, for text with named arguments (`"Item {number}"`), so a language can reorder
/// them.
macro_rules! formats {
    ($( $name:ident($arg:ident) { en: $en:literal, pt_br: $pt_br:literal $(,)? } )*) => {
        $(
            pub fn $name($arg: impl ::std::fmt::Display) -> String {
                match $crate::i18n::current() {
                    $crate::i18n::Language::English => format!($en),
                    $crate::i18n::Language::PortugueseBrazil => format!($pt_br),
                }
            }
        )*
    };
}

pub mod app {
    strings! {
        title { en: "MyApp", pt_br: "MyApp" }
        toggle_theme { en: "Toggle theme", pt_br: "Alternar tema" }
    }
}

pub mod items {
    strings! {
        add_item { en: "Add item", pt_br: "Adicionar item" }
        empty_title { en: "No items yet", pt_br: "Nenhum item ainda" }
        empty_hint {
            en: "Add the first one to start your list.",
            pt_br: "Adicione o primeiro para começar a lista.",
        }
        loading { en: "Loading items...", pt_br: "Carregando itens..." }
        retry { en: "Retry", pt_br: "Tentar de novo" }
        remove_item { en: "Remove item", pt_br: "Remover item" }
        mark_done { en: "Mark as done", pt_br: "Marcar como feito" }
        mark_not_done { en: "Mark as not done", pt_br: "Marcar como não feito" }
    }

    formats! {
        untitled_item(number) { en: "Item {number}", pt_br: "Item {number}" }
    }
}

pub mod problems {
    use myapp_core::Problem;
    use myapp_core::domain::DomainError;

    strings! {
        empty_title { en: "The title cannot be empty.", pt_br: "O título não pode ficar vazio." }
        title_too_long { en: "The title is too long.", pt_br: "O título é longo demais." }
        unknown_item { en: "That item no longer exists.", pt_br: "Esse item não existe mais." }
        duplicate_item {
            en: "The saved items repeat an id.",
            pt_br: "Os itens salvos repetem um id.",
        }
        store_unavailable {
            en: "Your items could not be saved or loaded. Try again.",
            pt_br: "Não foi possível salvar ou carregar seus itens. Tente de novo.",
        }
        store_corrupt {
            en: "The saved items could not be read.",
            pt_br: "Não foi possível ler os itens salvos.",
        }
    }

    /// The message a person reads for a `Problem`.
    pub fn describe(problem: &Problem) -> &'static str {
        match problem {
            Problem::Invalid(DomainError::EmptyTitle) => empty_title(),
            Problem::Invalid(DomainError::TitleTooLong { .. }) => title_too_long(),
            Problem::Invalid(DomainError::UnknownItem(_)) => unknown_item(),
            Problem::Invalid(DomainError::DuplicateItem(_)) => duplicate_item(),
            Problem::StoreUnavailable => store_unavailable(),
            Problem::StoreCorrupt => store_corrupt(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_then_current_round_trips() {
        for language in Language::ALL {
            set(language);
            assert_eq!(current(), language);
        }
        set(Language::English);
    }

    #[test]
    fn language_tags_parse() {
        assert_eq!(Language::from_tag("en"), Some(Language::English));
        assert_eq!(
            Language::from_tag("pt-BR"),
            Some(Language::PortugueseBrazil)
        );
        assert_eq!(Language::from_tag("pt"), Some(Language::PortugueseBrazil));
        assert_eq!(Language::from_tag("fr"), None);
    }
}
