//! Candidate-neutral registry used by the shared harness baseline.

use std::path::PathBuf;

use cityjson_lib::{Error, Result};

use super::{VertexStore, VertexStoreStrategy};

/// The shared baseline deliberately has no active representation.
pub const ACTIVE_STRATEGY: Option<VertexStoreStrategy> = None;

/// Candidate branches replace this function with their strategy factory.
///
/// # Errors
///
/// Returns an error on the candidate-neutral harness branch.
pub fn create(_sidecar_path: PathBuf) -> Result<Box<dyn VertexStore>> {
    Err(Error::Import(
        "this branch contains the candidate-neutral harness; select a candidate branch".into(),
    ))
}
