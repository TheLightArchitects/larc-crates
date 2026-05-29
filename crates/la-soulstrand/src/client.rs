use crate::{SoulstrandError, RetrievalResult};

/// Client for knowledge graph operations.
#[derive(Debug)]
pub struct SoulClient {
    // Will be filled by builder pattern
    _inner: (),
}

impl SoulClient {
    /// Create a new builder.
    pub fn builder() -> crate::SoulClientBuilder {
        crate::SoulClientBuilder::default()
    }

    /// Retrieve steps matching a query, returning top-k results.
    pub async fn retrieve(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<RetrievalResult>, SoulstrandError> {
        // Implementation depends on backend (sqlite or helix)
        todo!("implement with backend")
    }
}