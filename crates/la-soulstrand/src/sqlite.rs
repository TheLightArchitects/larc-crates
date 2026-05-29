/// SQLite backend for Tier 1 retrieval (BM25/FTS5).
///
/// No Neo4j required — works with local SQLite databases.
use crate::{RetrievalResult, SoulstrandError};

/// SQLite-backed knowledge graph storage.
pub struct SqliteBackend {
    // Will hold rusqlite::Connection when implemented
    _inner: (),
}

impl SqliteBackend {
    /// Open a SQLite database at the given path.
    pub fn open(_path: &str) -> Result<Self, SoulstrandError> {
        todo!("implement SQLite backend")
    }

    /// Retrieve steps using BM25/FTS5.
    pub fn retrieve_bm25(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<RetrievalResult>, SoulstrandError> {
        todo!("implement BM25 retrieval")
    }
}
