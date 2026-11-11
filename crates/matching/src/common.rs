use types::order::OrderError;

use book::BookError;

use thiserror::Error;

// ── Matching Error ────────────────────────────────────────────────────────────

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MatchingError {
    #[error(transparent)]
    OrderError(#[from] OrderError),

    #[error(transparent)]
    BookError(#[from] BookError),
}
