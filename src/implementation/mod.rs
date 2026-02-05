//! Implementation details for the Commerce plugin

pub mod cart_system;
pub mod inventory_sync;
pub mod order_management;
pub mod product_catalog;

use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use essentia_traits::plugin_contracts::flexforge_integration::{
    ConfigField, ConfigSchema, FlexForgeIntegration, FlexForgePanelCategory, UiConfigurable,
};

/// `FlexForge` integration for the Commerce plugin
#[derive(Debug)]
pub struct CommerceFlexForgeIntegration {
    config: Arc<Mutex<super::types::CommerceConfig>>,
}

impl CommerceFlexForgeIntegration {
    /// Create a new `FlexForge` integration instance
    #[must_use]
    pub fn new() -> Self {
        Self { config: Arc::new(Mutex::new(super::types::CommerceConfig::default())) }
    }

    fn config(&self) -> super::types::CommerceConfig {
        self.config.lock().map(|c| c.clone()).unwrap_or_default()
    }

    fn set_config(&self, config: super::types::CommerceConfig) {
        if let Ok(mut guard) = self.config.lock() {
            *guard = config;
        }
    }
}

impl Default for CommerceFlexForgeIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl FlexForgeIntegration for CommerceFlexForgeIntegration {
    fn panel_id(&self) -> &str {
        "commerce_config"
    }

    fn category(&self) -> FlexForgePanelCategory {
        FlexForgePanelCategory::System
    }

    fn display_name(&self) -> &str {
        "Commerce"
    }

    fn on_panel_activate(&mut self) {}

    fn on_panel_deactivate(&mut self) {}
}

impl UiConfigurable for CommerceFlexForgeIntegration {
    fn config_schema(&self) -> ConfigSchema {
        ConfigSchema::new()
            .with_field(ConfigField::toggle(
                "marketplace_enabled",
                "Marketplace",
                true,
            ))
            .with_field(ConfigField::toggle(
                "affiliate_enabled",
                "Affiliate System",
                true,
            ))
            .with_field(ConfigField::select("currency", "Currency", vec![
                "ESS".to_string(),
                "BTC".to_string(),
                "ETH".to_string(),
                "USDT".to_string(),
            ]))
            .with_field(ConfigField::number(
                "fee_percentage",
                "Fee %",
                2.5,
                0.0,
                10.0,
            ))
            .with_field(ConfigField::toggle("genesis_sync", "Genesis Sync", true))
            .with_field(ConfigField::toggle("auto_verify", "Auto-Verify", false))
    }

    fn on_config_changed(&mut self, key: &str, value: &str) -> Result<(), String> {
        let mut config = self.config();
        match key {
            "marketplace_enabled" => config.marketplace_enabled = value == "true",
            "affiliate_enabled" => config.affiliate_enabled = value == "true",
            "currency" => config.currency = value.to_string(),
            "fee_percentage" => {
                config.fee_percentage = value.parse().map_err(|_| "Invalid number")?;
            },
            "genesis_sync" => config.genesis_sync = value == "true",
            "auto_verify" => config.auto_verify = value == "true",
            _ => return Err(format!("Unknown key: {}", key)),
        }
        self.set_config(config);
        Ok(())
    }

    fn apply_config(&mut self, config: &[(String, String)]) -> Result<(), String> {
        for (key, value) in config {
            self.on_config_changed(key, value)?;
        }
        Ok(())
    }

    fn get_current_config(&self) -> Vec<(String, String)> {
        let config = self.config();
        vec![
            (
                "marketplace_enabled".to_string(),
                config.marketplace_enabled.to_string(),
            ),
            (
                "affiliate_enabled".to_string(),
                config.affiliate_enabled.to_string(),
            ),
            ("currency".to_string(), config.currency),
            (
                "fee_percentage".to_string(),
                config.fee_percentage.to_string(),
            ),
            ("genesis_sync".to_string(), config.genesis_sync.to_string()),
            ("auto_verify".to_string(), config.auto_verify.to_string()),
        ]
    }

    fn reset_to_defaults(&mut self) {
        self.set_config(super::types::CommerceConfig::default());
    }
}

#[cfg(all(test, feature = "full-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let integration = CommerceFlexForgeIntegration::new();
        let config = integration.config();
        assert!(config.marketplace_enabled);
        assert_eq!(config.currency, "ESS");
    }
}
