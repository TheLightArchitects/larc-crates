//! # larc-benchmark
//!
//! Domain-agnostic performance benchmark trait definitions for knowledge retrieval systems.
//!
//! Implement [`BenchmarkSuite`] to define a benchmark for your retrieval system.
//! Use [`BenchmarkReport`], [`Metric`], and [`MetricValue`] to structure results.
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
//! ## Datasets
//!
//! Enable the `longmemeval` feature to use the `LongMemEval` dataset types.
//!
//! ```toml
//! [dependencies]
//! larc-benchmark = { version = "0.1", features = ["longmemeval"] }
//! ```

mod error;
mod report;

pub use error::BenchmarkError;
pub use report::{BenchmarkReport, Metric, MetricValue};

/// Core trait for benchmark suites.
///
/// Implement to define a benchmark for your retrieval system.
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
