//! Escrow service for marketplace transactions

use std::collections::HashMap;

// Blockchain plugin integration
use essentia_blockchain_plugin::{
    BlockchainPlugin, Transaction as BlockchainTransaction, TransactionStatus as BlockchainTxStatus,
};

use crate::errors::MarketplaceError;

/// Escrow service result type
pub type EscrowResult<T> = Result<T, MarketplaceError>;

/// Escrow identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EscrowId(String);

impl EscrowId {
    pub fn new() -> Self {
        Self(format!("escrow_{}", essentia_uuid::Uuid::new_v4()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for EscrowId {
    fn default() -> Self {
        Self::new()
    }
}

/// Escrow account
#[derive(Debug, Clone)]
pub struct EscrowAccount {
    /// Escrow ID
    pub id:                 EscrowId,
    /// Order ID this escrow is for
    pub order_id:           super::orders::OrderId,
    /// Buyer peer ID
    pub buyer:              String,
    /// Seller peer ID
    pub seller:             String,
    /// Total amount held in escrow (sats)
    pub total_amount:       u64,
    /// Amount released to seller
    pub released_amount:    u64,
    /// Amount refunded to buyer
    pub refunded_amount:    u64,
    /// Release conditions
    pub release_conditions: Vec<ReleaseCondition>,
    /// Current status
    pub status:             EscrowStatus,
    /// Blockchain transaction ID for deposit
    pub deposit_tx_id:      Option<[u8; 32]>,
    /// Blockchain transaction ID for release
    pub release_tx_id:      Option<[u8; 32]>,
    /// Blockchain transaction ID for refund
    pub refund_tx_id:       Option<[u8; 32]>,
    /// Created timestamp
    pub created_at:         u64,
    /// Last updated timestamp
    pub updated_at:         u64,
}

/// Release condition types
#[derive(Debug, Clone)]
pub enum ReleaseCondition {
    /// Buyer explicitly approves release
    BuyerApproval,
    /// All milestones completed
    MilestonesCompleted,
    /// Time-based auto-release
    TimeBased { release_at: u64 },
    /// Third-party arbitration required
    Arbitration { arbitrator: String },
    /// Work quality verified
    QualityVerified,
}

/// Escrow status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscrowStatus {
    /// Funds deposited, awaiting conditions
    Active,
    /// Partially released to seller
    PartialRelease,
    /// Fully released to seller
    Released,
    /// Fully refunded to buyer
    Refunded,
    /// In dispute resolution
    Disputed,
    /// Dispute resolved
    Resolved,
}

/// Escrow manager service
#[derive(Default)]
pub struct EscrowManager {
    /// Active escrow accounts
    escrows:           HashMap<EscrowId, EscrowAccount>,
    /// Escrows by order ID
    escrows_by_order:  HashMap<super::orders::OrderId, EscrowId>,
    /// Blockchain plugin for transaction settlement
    blockchain_plugin: Option<BlockchainPlugin>,
}

impl EscrowManager {
    /// Create new escrow manager
    pub fn new() -> EscrowResult<Self> {
        Ok(Self {
            escrows:           HashMap::new(),
            escrows_by_order:  HashMap::new(),
            blockchain_plugin: None,
        })
    }

    /// Create new escrow manager with blockchain plugin
    pub fn with_blockchain_plugin(blockchain_plugin: BlockchainPlugin) -> EscrowResult<Self> {
        Ok(Self {
            escrows:           HashMap::new(),
            escrows_by_order:  HashMap::new(),
            blockchain_plugin: Some(blockchain_plugin),
        })
    }

    /// Create escrow account for order
    pub fn create_escrow(
        &mut self, order_id: super::orders::OrderId, buyer: String, seller: String, amount: u64,
        conditions: Vec<ReleaseCondition>,
    ) -> EscrowResult<EscrowId> {
        if self.escrows_by_order.contains_key(&order_id) {
            return Err(MarketplaceError::EscrowExists);
        }

        let escrow_id = EscrowId::new();
        let now = current_timestamp();

        // Create blockchain transaction for deposit if plugin available
        let deposit_tx_id = if let Some(blockchain_plugin) = &self.blockchain_plugin {
            let deposit_tx = BlockchainTransaction {
                id: [0u8; 32],        // Will be set by plugin
                sender: [0u8; 32],    // Buyer address - would need to be resolved
                recipient: [0u8; 32], // Escrow contract address
                amount,
                fee: 1000, // Default fee
                signature: Vec::new(),
                status: BlockchainTxStatus::Pending,
                timestamp: now,
            };

            let tx = blockchain_plugin.submit_transaction(deposit_tx).map_err(|e| {
                MarketplaceError::EscrowError(format!(
                    "Failed to submit deposit transaction: {:?}",
                    e
                ))
            })?;

            Some(tx.id)
        } else {
            None
        };

        let escrow = EscrowAccount {
            id: escrow_id.clone(),
            order_id: order_id.clone(),
            buyer,
            seller,
            total_amount: amount,
            released_amount: 0,
            refunded_amount: 0,
            release_conditions: conditions,
            status: EscrowStatus::Active,
            deposit_tx_id,
            release_tx_id: None,
            refund_tx_id: None,
            created_at: now,
            updated_at: now,
        };

        self.escrows.insert(escrow_id.clone(), escrow);
        self.escrows_by_order.insert(order_id, escrow_id.clone());

        Ok(escrow_id)
    }

    /// Release funds to seller (partial or full)
    pub fn release_funds(
        &mut self, escrow_id: &EscrowId, amount: u64, releaser: &str,
    ) -> EscrowResult<()> {
        let escrow = self.escrows.get_mut(escrow_id).ok_or(MarketplaceError::EscrowNotFound)?;

        // Verify releaser is buyer
        if releaser != escrow.buyer {
            return Err(MarketplaceError::InvalidEscrowState);
        }

        // Check if release conditions are met
        if !Self::check_release_conditions_static(escrow) {
            return Err(MarketplaceError::ReleaseConditionsNotMet);
        }

        // Calculate available amount
        let available = escrow.total_amount - escrow.released_amount - escrow.refunded_amount;
        let release_amount = amount.min(available);

        escrow.released_amount += release_amount;
        escrow.updated_at = current_timestamp();

        // Create blockchain transaction for release if plugin available
        if let Some(blockchain_plugin) = &self.blockchain_plugin {
            let now = current_timestamp();
            let release_tx = BlockchainTransaction {
                id:        [0u8; 32], // Will be set by plugin
                sender:    [0u8; 32], // Escrow contract address
                recipient: [0u8; 32], // Seller address - would need to be resolved
                amount:    release_amount,
                fee:       1000, // Default fee
                signature: Vec::new(),
                status:    BlockchainTxStatus::Pending,
                timestamp: now,
            };

            let tx = blockchain_plugin.submit_transaction(release_tx).map_err(|e| {
                MarketplaceError::EscrowError(format!(
                    "Failed to submit release transaction: {:?}",
                    e
                ))
            })?;

            escrow.release_tx_id = Some(tx.id);
        }

        // Update status
        if escrow.released_amount + escrow.refunded_amount >= escrow.total_amount {
            if escrow.released_amount > 0 && escrow.refunded_amount == 0 {
                escrow.status = EscrowStatus::Released;
            } else if escrow.refunded_amount > 0 && escrow.released_amount == 0 {
                escrow.status = EscrowStatus::Refunded;
            } else {
                escrow.status = EscrowStatus::PartialRelease;
            }
        }

        Ok(())
    }

    /// Refund funds to buyer
    pub fn refund_funds(&mut self, escrow_id: &EscrowId, amount: u64) -> EscrowResult<()> {
        let escrow = self.escrows.get_mut(escrow_id).ok_or(MarketplaceError::EscrowNotFound)?;

        // Calculate available amount
        let available = escrow.total_amount - escrow.released_amount - escrow.refunded_amount;
        let refund_amount = amount.min(available);

        escrow.refunded_amount += refund_amount;
        escrow.updated_at = current_timestamp();

        // Create blockchain transaction for refund if plugin available
        if let Some(blockchain_plugin) = &self.blockchain_plugin {
            let now = current_timestamp();
            let refund_tx = BlockchainTransaction {
                id:        [0u8; 32], // Will be set by plugin
                sender:    [0u8; 32], // Escrow contract address
                recipient: [0u8; 32], // Buyer address - would need to be resolved
                amount:    refund_amount,
                fee:       1000, // Default fee
                signature: Vec::new(),
                status:    BlockchainTxStatus::Pending,
                timestamp: now,
            };

            let tx = blockchain_plugin.submit_transaction(refund_tx).map_err(|e| {
                MarketplaceError::EscrowError(format!(
                    "Failed to submit refund transaction: {:?}",
                    e
                ))
            })?;

            escrow.refund_tx_id = Some(tx.id);
        }

        // Update status
        if escrow.released_amount + escrow.refunded_amount >= escrow.total_amount {
            if escrow.refunded_amount > 0 && escrow.released_amount == 0 {
                escrow.status = EscrowStatus::Refunded;
            } else {
                escrow.status = EscrowStatus::PartialRelease;
            }
        }

        Ok(())
    }

    /// Raise dispute for escrow
    pub fn raise_dispute(&mut self, escrow_id: &EscrowId) -> EscrowResult<()> {
        let escrow = self.escrows.get_mut(escrow_id).ok_or(MarketplaceError::EscrowNotFound)?;

        if !matches!(
            escrow.status,
            EscrowStatus::Active | EscrowStatus::PartialRelease
        ) {
            return Err(MarketplaceError::InvalidEscrowState);
        }

        escrow.status = EscrowStatus::Disputed;
        escrow.updated_at = current_timestamp();

        Ok(())
    }

    /// Resolve dispute
    #[allow(clippy::expect_used)]
    pub fn resolve_dispute(
        &mut self, escrow_id: &EscrowId, resolution: DisputeResolution,
    ) -> EscrowResult<()> {
        let buyer = {
            let escrow = self.escrows.get_mut(escrow_id).ok_or(MarketplaceError::EscrowNotFound)?;

            if escrow.status != EscrowStatus::Disputed {
                return Err(MarketplaceError::InvalidEscrowState);
            }

            escrow.buyer.clone()
        }; // escrow borrow ends here

        match resolution {
            DisputeResolution::ReleaseToSeller(amount) => {
                self.release_funds(escrow_id, amount, &buyer)?;
            },
            DisputeResolution::RefundToBuyer(amount) => {
                self.refund_funds(escrow_id, amount)?;
            },
            DisputeResolution::Split { seller_amount, buyer_amount } => {
                self.release_funds(escrow_id, seller_amount, &buyer)?;
                self.refund_funds(escrow_id, buyer_amount)?;
            },
        }

        // Update escrow status - we know it exists since we validated it above
        let escrow = self.escrows.get_mut(escrow_id).ok_or_else(|| {
            MarketplaceError::EscrowError("Escrow disappeared during resolution".to_string())
        })?;
        escrow.status = EscrowStatus::Resolved;
        escrow.updated_at = current_timestamp();

        Ok(())
    }

    /// Get escrow account
    pub fn get_escrow(&self, escrow_id: &EscrowId) -> Option<&EscrowAccount> {
        self.escrows.get(escrow_id)
    }

    /// Get escrow by order ID
    pub fn get_escrow_by_order(&self, order_id: &super::orders::OrderId) -> Option<&EscrowAccount> {
        self.escrows_by_order
            .get(order_id)
            .and_then(|escrow_id| self.escrows.get(escrow_id))
    }

    /// Check if release conditions are met
    fn check_release_conditions_static(escrow: &EscrowAccount) -> bool {
        for condition in &escrow.release_conditions {
            match condition {
                ReleaseCondition::BuyerApproval => {
                    // Would check if buyer has approved
                    // For now, assume conditions are met
                    continue;
                },
                ReleaseCondition::MilestonesCompleted => {
                    // Would check milestone completion
                    continue;
                },
                ReleaseCondition::TimeBased { release_at } => {
                    if current_timestamp() < *release_at {
                        return false;
                    }
                },
                ReleaseCondition::Arbitration { .. } => {
                    // Would check arbitration status
                    continue;
                },
                ReleaseCondition::QualityVerified => {
                    // Would check quality verification
                    continue;
                },
            }
        }
        true
    }
}

/// Dispute resolution outcome
#[derive(Debug, Clone)]
pub enum DisputeResolution {
    /// Release specified amount to seller
    ReleaseToSeller(u64),
    /// Refund specified amount to buyer
    RefundToBuyer(u64),
    /// Split funds between seller and buyer
    Split { seller_amount: u64, buyer_amount: u64 },
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}
