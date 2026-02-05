//! Type definitions for the Commerce plugin

use std::fmt::Debug;

use essentia_api::implementation::commerce::BusinessEntity;

use crate::errors::CommerceError;

/// Genesis Directory Node for commerce operations
#[derive(Debug, Clone)]
pub struct GenesisDirectory {
    /// Registered business entities
    pub entities: Vec<BusinessEntity>,
}

impl GenesisDirectory {
    /// Create a new genesis directory
    #[must_use]
    pub fn new() -> Self {
        Self { entities: Vec::new() }
    }

    /// Register a business entity with coherence validation
    ///
    /// # Errors
    ///
    /// Returns `CommerceError::ValidationError` if the entity's coherence score
    /// is below the required threshold of 0.99.
    pub fn register_business(&mut self, entity: BusinessEntity) -> Result<(), CommerceError> {
        // Validate coherence score meets threshold
        if entity.coherence_score < 0.99 {
            return Err(CommerceError::ValidationError(format!(
                "Business entity coherence score {} below required threshold 0.99",
                entity.coherence_score
            )));
        }

        self.entities.push(entity);
        Ok(())
    }

    /// Query business entities
    pub fn query(&self, filter: impl Fn(&BusinessEntity) -> bool) -> Vec<&BusinessEntity> {
        self.entities.iter().filter(|e| filter(e)).collect()
    }
}

impl Default for GenesisDirectory {
    fn default() -> Self {
        Self::new()
    }
}

/// Commerce configuration for `FlexForge` panel
#[derive(Debug, Clone)]
pub struct CommerceConfig {
    /// Enable marketplace functionality
    pub marketplace_enabled: bool,
    /// Enable affiliate program
    pub affiliate_enabled:   bool,
    /// Default currency for transactions
    pub currency:            String,
    /// Transaction fee percentage
    pub fee_percentage:      f64,
    /// Enable genesis synchronization
    pub genesis_sync:        bool,
    /// Enable automatic verification
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

pub mod inventory_sync;
pub mod product_catalog;
