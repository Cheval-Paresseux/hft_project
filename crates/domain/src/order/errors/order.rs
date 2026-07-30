use super::modification::ModificationError;
use super::fill::FillError;
use thiserror::Error;

// ── Order Error ───────────────────────────────────────────────────────────────

#[derive(Error, Debug, PartialEq, Eq)]
pub enum OrderError {
    #[error(transparent)]
    InvalidModification(#[from] ModificationError),

    #[error(transparent)]
    InvalidFill(#[from] FillError),
}

