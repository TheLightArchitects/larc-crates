use crate::SoulstrandError;
use crate::SoulClient;

/// Builder for [`SoulClient`].
#[derive(Debug, Default)]
pub struct SoulClientBuilder {
    // Backend configuration will be added per feature gate
    _inner: (),
}

impl SoulClientBuilder {
    /// Connect using default configuration.
    pub async fn connect(self) -> Result<SoulClient, SoulstrandError> {
        // Implementation depends on backend (sqlite or helix)
        todo!("implement with backend")
    }
}