//! `EmbeddingBackend` — contract for text embedding models.
//!
//! Implement this to plug in any embedding provider:
//! fastembed (SDK default), OpenAI `text-embedding-3-small`, Ollama, etc.

use async_trait::async_trait;
use std::fmt::Debug;

use crate::SoulvaultError;

/// Contract for a text embedding model.
///
/// The production implementation in `lightarchitects-sdk` uses `fastembed`.
/// Implement this trait to substitute any embedding provider.
///
/// # Example
///
/// ```rust,ignore
/// use la_soulvault::{EmbeddingBackend, SoulvaultError};
///
/// struct OllamaEmbedder { model: String, base_url: String }
///
/// #[async_trait::async_trait]
/// impl EmbeddingBackend for OllamaEmbedder {
///     async fn embed(&self, text: &str) -> Result<Vec<f32>, SoulvaultError> {
///         // POST /api/embeddings
///         todo!()
///     }
///     async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, SoulvaultError> {
///         let mut out = Vec::with_capacity(texts.len());
///         for t in texts { out.push(self.embed(t).await?); }
///         Ok(out)
///     }
///     fn dimensions(&self) -> usize { 768 }
/// }
/// ```
#[async_trait]
pub trait EmbeddingBackend: Debug + Send + Sync {
    /// Embed a single text string into a dense vector.
    async fn embed(&self, text: &str) -> Result<Vec<f32>, SoulvaultError>;

    /// Embed a batch of texts — implementations should parallelise internally.
    ///
    /// The default implementation calls [`embed`](Self::embed) sequentially;
    /// override for performance.
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, SoulvaultError> {
        let mut out = Vec::with_capacity(texts.len());
        for text in texts {
            out.push(self.embed(text).await?);
        }
        Ok(out)
    }

    /// Dimensionality of the output vectors — needed to size vector indices.
    fn dimensions(&self) -> usize;
}
