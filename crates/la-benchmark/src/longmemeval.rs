use crate::{BenchmarkError, BenchmarkReport, BenchmarkSuite};

/// A document chunk for indexing.
#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: serde_json::Value,
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
#[derive(Debug, Clone)]
pub struct LongMemEvalDataset {
    pub questions: Vec<LongMemEvalQuestion>,
    pub corpus_size: usize,
}

/// A single LongMemEval question with ground truth.
#[derive(Debug, Clone)]
pub struct LongMemEvalQuestion {
    pub query: String,
    pub ground_truth_ids: Vec<String>,
    pub category: String,
}

/// A retrieval result from LongMemEval.
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    pub id: String,
    pub score: f64,
    pub content: String,
}
