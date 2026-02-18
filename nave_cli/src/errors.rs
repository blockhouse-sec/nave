use acvm::FieldElement;
use nargo::{NargoError, errors::CompileError};
use nargo_toml::ManifestError;
use noirc_abi::errors::AbiError;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum CliError {
    #[error("{0}")]
    Generic(String),

    /// Artifact CLI error
    #[error(transparent)]
    ArtifactError(#[from] noir_artifact_cli::errors::CliError),

    /// ABI encoding/decoding error
    #[error(transparent)]
    AbiError(#[from] AbiError),

    /// Error from Nargo
    #[error(transparent)]
    NargoError(#[from] NargoError<FieldElement>),

    /// Error from Manifest
    #[error(transparent)]
    ManifestError(#[from] ManifestError),

    /// Error from the compilation pipeline
    #[error(transparent)]
    CompileError(#[from] CompileError),
}
