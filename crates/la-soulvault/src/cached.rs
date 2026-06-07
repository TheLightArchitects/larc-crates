//! Moka-backed TTL cache wrapping any [`EmbeddingBackend`].
//!
//! # Cache hit path (~0µs for cached texts)
//!
//! `SHA-256(text \x00 provider_name)` → `Cache::get()` → cached `Vec<f32>`
//!
//! # Cache miss path
//!
//! Pass-through to inner backend; populate cache on return.
//!
//! # When this helps
//!
//! Repeated queries with identical text hit the cache — e.g. the same helix
//! title appearing across multiple search sessions, or repeated embedding of
//! shared terminology. Semantic caching literature reports 40–70% hit rates
//! for knowledge-base retrieval workloads. With `nomic-embed-text` at ~50ms
//! per call, a 50% hit rate saves ~25ms per query at zero accuracy cost.
//!
//! # Cache coherence
//!
//! Text embeddings are deterministic for a fixed model version: the same text
//! always produces the same vector. No eviction is needed for correctness —
//! only the TTL ensures memory is bounded. On model upgrade, replace or
//! re-create the [`CachedEmbeddingProvider`]; all entries will TTL out.
//!
//! # Feature
//!
//! Gated behind `embedding-cache` (implies `embedding`).

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use moka::future::Cache;
use sha2::{Digest, Sha256};

use crate::{EmbeddingBackend, SoulvaultError};

/// Maximum number of cached embedding vectors.
const CACHE_MAX_ENTRIES: u64 = 4_096;

/// Default TTL for cached embeddings. Deterministic per model version → safe
/// for long TTLs; 5 minutes keeps memory bounded.
const CACHE_TTL_SECS: u64 = 300;

/// Wraps an [`EmbeddingBackend`] with a per-text moka TTL cache.
///
/// Thread-safe: `Clone`able, backed by `moka::future::Cache` (`Send + Sync`).
pub struct CachedEmbeddingProvider {
    inner: Arc<dyn EmbeddingBackend>,
    cache: Cache<[u8; 32], Arc<Vec<f32>>>,
    name: &'static str,
}

impl fmt::Debug for CachedEmbeddingProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CachedEmbeddingProvider")
            .field("name", &self.name)
            .field("max_capacity", &CACHE_MAX_ENTRIES)
            .finish_non_exhaustive()
    }
}

impl CachedEmbeddingProvider {
    /// Wrap `inner` with a default 5-minute, 4096-entry cache.
    pub fn new(inner: Arc<dyn EmbeddingBackend>, name: &'static str) -> Self {
        Self::with_config(inner, name, CACHE_MAX_ENTRIES, Duration::from_secs(CACHE_TTL_SECS))
    }

    /// Wrap `inner` with custom capacity and TTL.
    pub fn with_config(
        inner: Arc<dyn EmbeddingBackend>,
        name: &'static str,
        max_entries: u64,
        ttl: Duration,
    ) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_entries)
            .time_to_live(ttl)
            .build();
        Self { inner, cache, name }
    }

    /// SHA-256(text \x00 `name`) — stable key per text+model combination.
    fn cache_key(text: &str, name: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        hasher.update(b"\x00");
        hasher.update(name.as_bytes());
        hasher.finalize().into()
    }
}

#[async_trait]
impl EmbeddingBackend for CachedEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, SoulvaultError> {
        let key = Self::cache_key(text, self.name);
        if let Some(cached) = self.cache.get(&key).await {
            return Ok((*cached).clone());
        }
        let vec = self.inner.embed(text).await?;
        self.cache.insert(key, Arc::new(vec.clone())).await;
        Ok(vec)
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, SoulvaultError> {
        let mut results: Vec<Option<Vec<f32>>> = vec![None; texts.len()];
        let mut missed_indices: Vec<usize> = Vec::new();
        let mut missed_texts: Vec<&str> = Vec::new();

        // Phase 1: populate from cache.
        for (i, &text) in texts.iter().enumerate() {
            let key = Self::cache_key(text, self.name);
            if let Some(cached) = self.cache.get(&key).await {
                results[i] = Some((*cached).clone());
            } else {
                missed_indices.push(i);
                missed_texts.push(text);
            }
        }

        if missed_texts.is_empty() {
            return Ok(results.into_iter().map(Option::unwrap_or_default).collect());
        }

        // Phase 2: fetch misses from inner backend and populate cache.
        let embeddings = self.inner.embed_batch(&missed_texts).await?;
        for (offset, (&orig_i, embedding)) in missed_indices.iter().zip(embeddings).enumerate() {
            let key = Self::cache_key(missed_texts[offset], self.name);
            self.cache.insert(key, Arc::new(embedding.clone())).await;
            results[orig_i] = Some(embedding);
        }

        results
            .into_iter()
            .enumerate()
            .map(|(i, v)| {
                v.ok_or_else(|| {
                    SoulvaultError::Backend(format!("missing embedding result at index {i}"))
                })
            })
            .collect()
    }

    fn dimensions(&self) -> usize {
        self.inner.dimensions()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Counting mock that records how many times embed() is called.
    #[derive(Debug)]
    struct CountingBackend {
        calls: Arc<AtomicUsize>,
        dim: usize,
    }

    impl CountingBackend {
        fn new(dim: usize) -> (Self, Arc<AtomicUsize>) {
            let calls = Arc::new(AtomicUsize::new(0));
            (Self { calls: Arc::clone(&calls), dim }, calls)
        }
    }

    #[async_trait]
    impl EmbeddingBackend for CountingBackend {
        async fn embed(&self, _text: &str) -> Result<Vec<f32>, SoulvaultError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(vec![0.0; self.dim])
        }
        fn dimensions(&self) -> usize {
            self.dim
        }
    }

    #[tokio::test]
    async fn cache_hit_avoids_inner_call() {
        let (backend, calls) = CountingBackend::new(4);
        let provider = CachedEmbeddingProvider::new(Arc::new(backend), "test");

        provider.embed("hello").await.unwrap();
        provider.embed("hello").await.unwrap(); // cache hit
        assert_eq!(calls.load(Ordering::SeqCst), 1, "inner called only once");
    }

    #[tokio::test]
    async fn different_texts_get_different_keys() {
        let k1 = CachedEmbeddingProvider::cache_key("foo", "model");
        let k2 = CachedEmbeddingProvider::cache_key("bar", "model");
        assert_ne!(k1, k2);
    }

    #[tokio::test]
    async fn batch_partial_miss_works() {
        let (backend, calls) = CountingBackend::new(4);
        let provider = CachedEmbeddingProvider::new(Arc::new(backend), "test");

        // Warm "a".
        provider.embed("a").await.unwrap();
        // "a" is cached, "b" is a miss — inner called once more for "b".
        let results = provider.embed_batch(&["a", "b"]).await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
}
