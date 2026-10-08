//! Interface text. Every string a person reads comes from here, never a literal in a view.
//!
//! `strings!` and the `match` in each hand-written format require a translation for every
//! [`Language`], so adding a language makes the compiler list every missing string.

use std::sync::atomic::{AtomicU8, Ordering};

/// Interface languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Language {
    English,
    PortugueseBrazil,
}

impl Language {
    /// The language for a tag such as `en` or `pt-BR`.
    pub(crate) fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "en" => Some(Self::English),
            "pt-BR" | "pt-br" | "pt" => Some(Self::PortugueseBrazil),
            _ => None,
        }
    }
}

/// The `Language` discriminant of the language in use.
static CURRENT: AtomicU8 = AtomicU8::new(Language::English as u8);

/// Sets the language in use.
pub(crate) fn set(language: Language) {
    CURRENT.store(language as u8, Ordering::Relaxed);
}

/// The language in use.
pub(crate) fn current() -> Language {
    match CURRENT.load(Ordering::Relaxed) {
        value if value == Language::PortugueseBrazil as u8 => Language::PortugueseBrazil,
        _ => Language::English,
    }
}

macro_rules! strings {
    ($( $name:ident { en: $en:literal, pt_br: $pt_br:literal $(,)? } )*) => {
        $(
            pub(crate) fn $name() -> &'static str {
                match $crate::i18n::current() {
                    $crate::i18n::Language::English => $en,
                    $crate::i18n::Language::PortugueseBrazil => $pt_br,
                }
            }
        )*
    };
}

pub(crate) mod app {
    strings! {
        title { en: "MyApp", pt_br: "MyApp" }
        toggle_theme { en: "Toggle theme", pt_br: "Alternar tema" }
    }
}

pub(crate) mod items {
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

    /// The default title of the `number`-th item.
    pub(crate) fn untitled_item(number: usize) -> String {
        match super::current() {
            super::Language::English => format!("Item {number}"),
            super::Language::PortugueseBrazil => format!("Item {number}"),
        }
    }
}

pub(crate) mod problems {
    use myapp_core::Problem;
    use myapp_core::domain::DomainError;

    strings! {
        empty_title { en: "The title cannot be empty.", pt_br: "O título não pode ficar vazio." }
        title_too_long { en: "The title is too long.", pt_br: "O título é longo demais." }
        no_ids_left {
            en: "No more items can be added.",
            pt_br: "Não é possível adicionar mais itens.",
        }
        unknown_item { en: "That item no longer exists.", pt_br: "Esse item não existe mais." }
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
    pub(crate) fn describe(problem: &Problem) -> &'static str {
        match problem {
            Problem::Invalid(DomainError::EmptyTitle) => empty_title(),
            Problem::Invalid(DomainError::TitleTooLong) => title_too_long(),
            Problem::Invalid(DomainError::UnknownItem(_)) => unknown_item(),
            Problem::Invalid(DomainError::NoIdsLeft) => no_ids_left(),
            Problem::StoreUnavailable => store_unavailable(),
            Problem::StoreCorrupt => store_corrupt(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
