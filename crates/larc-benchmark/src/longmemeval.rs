use crate::{BenchmarkError, BenchmarkReport, BenchmarkSuite};
use serde::{Deserialize, Serialize};

/// A document chunk for indexing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: serde_json::Value,
}

impl Document {
    /// Create a new document.
    #[must_use]
    pub fn new(id: String, content: String, metadata: serde_json::Value) -> Self {
        Self {
            id,
            content,
            metadata,
        }
    }
}

/// LongMemEval benchmark trait.
///
/// Implement this trait alongside [`BenchmarkSuite`] to run
/// the LongMemEval evaluation against your retrieval system.
pub trait LongMemEval: BenchmarkSuite {
    /// Index a corpus of documents into the retrieval system.
    fn index_corpus(&self, documents: &[Document]) -> Result<(), BenchmarkError>;

    /// Retrieve top-k results for a query.
    fn retrieve(&self, query: &str, k: usize) -> Result<Vec<RetrievalResult>, BenchmarkError>;

    /// Evaluate against a LongMemEval dataset.
    fn evaluate(&self, dataset: &LongMemEvalDataset) -> Result<BenchmarkReport, BenchmarkError>;
}

/// A LongMemEval dataset for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LongMemEvalDataset {
    pub questions: Vec<LongMemEvalQuestion>,
    pub corpus_size: usize,
}

impl LongMemEvalDataset {
    /// Create a new dataset.
    #[must_use]
    pub fn new(questions: Vec<LongMemEvalQuestion>, corpus_size: usize) -> Self {
        Self {
            questions,
            corpus_size,
        }
    }
}

/// A single LongMemEval question with ground truth.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LongMemEvalQuestion {
    pub query: String,
    pub ground_truth_ids: Vec<String>,
    pub category: String,
}

impl LongMemEvalQuestion {
    /// Create a new question.
    #[must_use]
    pub fn new(query: String, ground_truth_ids: Vec<String>, category: String) -> Self {
        Self {
            query,
            ground_truth_ids,
            category,
        }
    }
}

/// A retrieval result from LongMemEval.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RetrievalResult {
    pub id: String,
    pub score: f64,
    pub content: String,
}

impl RetrievalResult {
    /// Create a new retrieval result.
    #[must_use]
    pub fn new(id: String, score: f64, content: String) -> Self {
        Self { id, score, content }
    }
}
