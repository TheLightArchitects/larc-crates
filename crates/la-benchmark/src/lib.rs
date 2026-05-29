//! # la-benchmark
//!
//! Domain-agnostic performance benchmark framework for knowledge retrieval systems.
//!
//! Provides the `BenchmarkSuite` trait and a runner that any system can implement.
//! Feature-gated benchmark suites (e.g., `longmemeval`) add dataset-specific traits.
//!
//! ## Quick start
//!
//! ```no_run
//! use la_benchmark::{BenchmarkSuite, BenchmarkRunner, BenchmarkReport};
//!
//! struct MyRetriever;
//!
//! impl BenchmarkSuite for MyRetriever {
//!     fn name(&self) -> &str { "my-retriever" }
//!
//!     fn run(&self) -> Result<BenchmarkReport, la_benchmark::BenchmarkError> {
//!         // Run your benchmarks here
//!         todo!()
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let suite = MyRetriever;
//! let report = BenchmarkRunner::new().run(&suite)?;
//! println!("{}", report.summary());
//! # Ok(())
//! # }
//! ```

mod error;
mod report;
mod runner;

pub use error::BenchmarkError;
pub use report::{BenchmarkReport, Metric, MetricValue};
pub use runner::BenchmarkRunner;

/// Core trait that all benchmark suites implement.
pub trait BenchmarkSuite {
    /// Human-readable name for this benchmark suite.
    fn name(&self) -> &str;

    /// Execute the benchmark and return a report.
    fn run(&self) -> Result<BenchmarkReport, BenchmarkError>;

    /// Retrieve the last completed report without re-running.
    fn report(&self) -> BenchmarkReport;
}

// Feature-gated: LongMemEval benchmark
#[cfg(feature = "longmemeval")]
mod longmemeval;

#[cfg(feature = "longmemeval")]
pub use longmemeval::{
    LongMemEval,
    LongMemEvalDataset,
    RetrievalResult,
};