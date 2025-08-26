use super::*;

pub struct OvernightConfigManager;

impl OvernightConfigManager {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn get_config(&self, _home_id: &str) -> Option<OvernightConfig> {
        None
    }
    
    pub async fn update_config(&self, _config: OvernightConfig) -> anyhow::Result<()> {
        Ok(())
    }
    
    pub async fn enable_for_home(&self, _home_id: &str) -> anyhow::Result<()> {
        Ok(())
    }
    
    pub async fn disable_for_home(&self, _home_id: &str) -> anyhow::Result<()> {
        Ok(())
    }
}
