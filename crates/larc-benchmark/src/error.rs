use thiserror::Error;

/// Errors for `larc-benchmark`.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum BenchmarkError {
    #[error("benchmark execution failed: {0}")]
    ExecutionFailed(String),

    #[error("dataset not found: {0}")]
    DatasetNotFound(String),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}
