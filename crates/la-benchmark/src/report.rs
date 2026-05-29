use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A completed benchmark report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub suite_name: String,
    pub metrics: Vec<Metric>,
    pub duration: Duration,
    pub timestamp: String,
}

impl BenchmarkReport {
    /// Human-readable summary.
    pub fn summary(&self) -> String {
        format!(
            "{}: {} metrics in {:?}",
            self.suite_name,
            self.metrics.len(),
            self.duration
        )
    }
}

/// A single named metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: MetricValue,
    pub description: String,
}

/// A metric value — scalar or distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
