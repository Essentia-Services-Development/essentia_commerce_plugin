//! Type definitions for the Commerce plugin

use std::fmt::Debug;

/// Genesis Directory Node for commerce operations
#[derive(Debug, Clone)]
pub struct GenesisDirectory {
    pub entities: Vec<essentia_api::commerce::BusinessEntity>,
}

impl GenesisDirectory {
    /// Create a new genesis directory
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    /// Register a business entity with coherence validation
    pub fn register_business(
        &mut self,
        entity: essentia_api::commerce::BusinessEntity,
    ) -> Result<(), essentia_error::EssentiaError> {
        // Validate coherence score meets threshold
        if entity.coherence_score < 0.99 {
            return Err(essentia_error::EssentiaError::CoherenceViolation {
                score: entity.coherence_score as f64,
                threshold: 0.99,
                concerns: vec!["Business entity coherence below required threshold".to_string()],
            });
        }

        self.entities.push(entity);
        Ok(())
    }

    /// Query business entities
    pub fn query(&self, filter: impl Fn(&essentia_api::commerce::BusinessEntity) -> bool) -> Vec<&essentia_api::commerce::BusinessEntity> {
        self.entities.iter().filter(|e| filter(e)).collect()
    }
}

impl Default for GenesisDirectory {
    fn default() -> Self {
        Self::new()
    }
}

/// Commerce configuration for FlexForge panel
#[derive(Debug, Clone)]
pub struct CommerceConfig {
    pub marketplace_enabled: bool,
    pub affiliate_enabled:   bool,
    pub currency:            String,
    pub fee_percentage:      f64,
    pub genesis_sync:        bool,
    pub auto_verify:         bool,
}

impl Default for CommerceConfig {
    fn default() -> Self {
        Self {
            marketplace_enabled: true,
            affiliate_enabled:   true,
            currency:            "ESS".to_string(),
            fee_percentage:      2.5,
            genesis_sync:        true,
            auto_verify:         false,
        }
    }
}
