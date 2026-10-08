//! What went wrong, as the UI needs to know it.

use myapp_domain::DomainError;

use crate::StoreError;

/// A failure a person can be told about.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Problem {
    /// A domain rule rejected the input.
    #[error(transparent)]
    Invalid(#[from] DomainError),
    /// The item store could not be reached.
    #[error("the item store is unavailable")]
    StoreUnavailable,
    /// The saved items could not be read.
    #[error("the saved items could not be read")]
    StoreCorrupt,
}

/// Result of a use case.
pub type Result<T> = std::result::Result<T, Problem>;

impl Problem {
    /// Whether doing the same thing again could succeed.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::StoreUnavailable | Self::StoreCorrupt)
    }
}

impl From<StoreError> for Problem {
    fn from(error: StoreError) -> Self {
        tracing::warn!(
            error = %error,
            source = ?std::error::Error::source(&error),
            "item store failed"
        );
        match error {
            StoreError::Io(_) => Self::StoreUnavailable,
            StoreError::Corrupt(_) => Self::StoreCorrupt,
        }
    }
}
