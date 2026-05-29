use crate::{BenchmarkError, BenchmarkReport, BenchmarkSuite};

/// Runner for executing benchmark suites.
#[derive(Debug, Default)]
pub struct BenchmarkRunner {
    /// Timeout per benchmark (None = no timeout).
    pub timeout: Option<std::time::Duration>,
}

impl BenchmarkRunner {
    /// Create a new runner with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a timeout per benchmark.
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Run a benchmark suite and return the report.
    pub fn run(&self, suite: &dyn BenchmarkSuite) -> Result<BenchmarkReport, BenchmarkError> {
        let start = std::time::Instant::now();
        let report = suite.run()?;
        let elapsed = start.elapsed();

        // Override duration with actual measured time
        Ok(BenchmarkReport {
            duration: elapsed,
            ..report
        })
    }
}
