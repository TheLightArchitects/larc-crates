use serde::{Deserialize, Serialize};

/// A completed benchmark report.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BenchmarkReport {
    pub suite_name: String,
    pub metrics: Vec<Metric>,
    /// Total duration in milliseconds.
    pub duration_ms: u64,
    pub timestamp: String,
}

impl BenchmarkReport {
    /// Create a new benchmark report.
    #[must_use]
    pub fn new(
        suite_name: String,
        metrics: Vec<Metric>,
        duration_ms: u64,
        timestamp: String,
    ) -> Self {
        Self {
            suite_name,
            metrics,
            duration_ms,
            timestamp,
        }
    }

    /// Duration as `std::time::Duration`.
    pub fn duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.duration_ms)
    }

    /// Human-readable summary.
    pub fn summary(&self) -> String {
        format!(
            "{}: {} metrics in {}ms",
            self.suite_name,
            self.metrics.len(),
            self.duration_ms
        )
    }
}

/// A single named metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Metric {
    pub name: String,
    pub value: MetricValue,
    pub description: String,
}

impl Metric {
    /// Create a new metric.
    #[must_use]
    pub fn new(name: String, value: MetricValue, description: String) -> Self {
        Self {
            name,
            value,
            description,
        }
    }
}

/// A metric value — scalar or distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum MetricValue {
    Scalar(f64),
    Distribution {
        mean: f64,
        std_dev: f64,
        min: f64,
        max: f64,
        samples: usize,
    },
}
