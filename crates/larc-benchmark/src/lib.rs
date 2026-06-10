//! # la-benchmark
//!
//! Domain-agnostic performance benchmark trait definitions for knowledge retrieval systems.
//!
//! This crate defines the **interfaces** (traits, types, errors) that the private
//! `lightarchitects-sdk` implements. Users can implement these traits themselves
//! or consume the SDK via git dependency.
//!
//! ## Quick start
//!
//! ```rust
//! use larc_benchmark::{BenchmarkSuite, BenchmarkError, BenchmarkReport};
//!
//! struct MyRetriever;
//!
//! impl BenchmarkSuite for MyRetriever {
//!     fn name(&self) -> &str { "my-retriever" }
//!
//!     fn run(&self) -> Result<BenchmarkReport, BenchmarkError> {
//!         // Implement your benchmark logic here
//!         todo!()
//!     }
//! }
//! ```
//!
//! ## Implementations
//!
//! The concrete runner and dataset implementations live in the `lightarchitects-sdk` crate.
//! To use the production runner:
//!
//! ```toml
//! [dependencies]
//! la-benchmark = { git = "https://github.com/TheLightArchitect/lightarchitects-sdk" }
//! lightarchitects = { git = "https://github.com/TheLightArchitect/lightarchitects-sdk", features = ["benchmark"] }
//! ```

mod error;
mod report;

pub use error::BenchmarkError;
pub use report::{BenchmarkReport, Metric, MetricValue};

/// Core trait that all benchmark suites implement.
///
/// Implement this trait to define a benchmark suite for your retrieval system.
/// The production runner lives in `lightarchitects-sdk`.
pub trait BenchmarkSuite {
    /// Human-readable name for this benchmark suite.
    fn name(&self) -> &str;

    /// Execute the benchmark and return a report.
    fn run(&self) -> Result<BenchmarkReport, BenchmarkError>;
}

// Feature-gated: LongMemEval benchmark
#[cfg(feature = "longmemeval")]
mod longmemeval;

#[cfg(feature = "longmemeval")]
pub use longmemeval::{
    Document, LongMemEval, LongMemEvalDataset, LongMemEvalQuestion, RetrievalResult,
};
