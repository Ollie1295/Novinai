use anyhow::Result;
use std::sync::Arc;

pub trait OvernightStorage: Send + Sync {
    fn store(&self, data: &str) -> Result<()>;
}

pub struct InMemoryStorage;

impl OvernightStorage for InMemoryStorage {
    fn store(&self, _data: &str) -> Result<()> {
        Ok(())
    }
}

pub struct OvernightStorageFactory;

impl OvernightStorageFactory {
    pub fn create_in_memory() -> Arc<dyn OvernightStorage> {
        Arc::new(InMemoryStorage)
    }
}
