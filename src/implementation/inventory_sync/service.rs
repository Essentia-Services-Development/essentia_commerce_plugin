//! # Inventory Service Implementation
//!
//! Implementation of the InventoryService for managing inventory operations.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use crate::{errors::CommerceError, types::product_catalog::ProductId, types::inventory_sync::*};
use essentia_time::Instant;

impl InventoryService {
    /// Creates a new inventory service.
    #[must_use]
    pub fn new() -> Self {
        let service = Self {
            levels:      Arc::new(Mutex::new(HashMap::new())),
            locations:   Arc::new(Mutex::new(HashMap::new())),
            adjustments: Arc::new(Mutex::new(Vec::new())),
            transfers:   Arc::new(Mutex::new(HashMap::new())),
            sources:     Arc::new(Mutex::new(HashMap::new())),
        };

        // Add default location
        let _ = service.add_location(InventoryLocation::warehouse(
            LocationId::default_warehouse(),
            "Main Warehouse",
        ));

        service
    }

    // ========================================================================
    // LOCATION MANAGEMENT
    // ========================================================================

    /// Adds a location.
    pub fn add_location(&self, location: InventoryLocation) -> Result<(), CommerceError> {
        let mut locations = self.locations.lock().map_err(|_| CommerceError::LockError)?;

        if locations.contains_key(&location.id) {
            return Err(CommerceError::LocationAlreadyExists(
                location.id.0.to_string(),
            ));
        }

        locations.insert(location.id.clone(), location);
        Ok(())
    }

    /// Gets a location.
    pub fn get_location(&self, id: &LocationId) -> Result<InventoryLocation, CommerceError> {
        let locations = self.locations.lock().map_err(|_| CommerceError::LockError)?;
        locations
            .get(id)
            .cloned()
            .ok_or_else(|| CommerceError::LocationNotFound(id.0.to_string()))
    }

    /// Gets all active locations.
    pub fn get_active_locations(&self) -> Result<Vec<InventoryLocation>, CommerceError> {
        let locations = self.locations.lock().map_err(|_| CommerceError::LockError)?;
        Ok(locations.values().filter(|l| l.is_active).cloned().collect())
    }

    // ========================================================================
    // INVENTORY LEVEL MANAGEMENT
    // ========================================================================

    /// Sets inventory level for a product at a location.
    pub fn set_inventory(
        &self, product_id: ProductId, location_id: LocationId, on_hand: i64,
        reason: impl Into<String>,
    ) -> Result<(), CommerceError> {
        let key = InventoryKey {
            product_id:  product_id.clone(),
            variant_id:  None,
            location_id: location_id.clone(),
        };

        let mut levels = self.levels.lock().map_err(|_| CommerceError::LockError)?;

        let previous_quantity = levels.get(&key).map(|l| l.on_hand).unwrap_or(0);

        let level = levels
            .entry(key)
            .or_insert_with(|| InventoryLevel::new(product_id.clone(), location_id.clone()));

        level.on_hand = on_hand;
        level.recalculate_available();

        // Record adjustment - move values since we don't need them after this
        let adjustment = InventoryAdjustment::new(
            product_id,
            location_id,
            AdjustmentType::Adjustment,
            on_hand - previous_quantity,
            previous_quantity,
            reason,
        );

        drop(levels);
        self.record_adjustment(adjustment)?;

        Ok(())
    }

    /// Gets inventory level for a product at a location.
    pub fn get_inventory(
        &self, product_id: &ProductId, location_id: &LocationId,
    ) -> Result<InventoryLevel, CommerceError> {
        let key = InventoryKey {
            product_id:  product_id.clone(),
            variant_id:  None,
            location_id: location_id.clone(),
        };

        let levels = self.levels.lock().map_err(|_| CommerceError::LockError)?;
        levels
            .get(&key)
            .cloned()
            .ok_or_else(|| CommerceError::InventoryNotFound(product_id.0.to_string()))
    }

    /// Gets total available quantity across all locations.
    pub fn get_total_available(&self, product_id: &ProductId) -> Result<i64, CommerceError> {
        let levels = self.levels.lock().map_err(|_| CommerceError::LockError)?;

        let total: i64 = levels
            .iter()
            .filter(|(k, _)| &k.product_id == product_id)
            .map(|(_, v)| v.available)
            .sum();

        Ok(total)
    }

    /// Gets inventory levels across all locations.
    pub fn get_all_inventory_for_product(
        &self, product_id: &ProductId,
    ) -> Result<Vec<InventoryLevel>, CommerceError> {
        let levels = self.levels.lock().map_err(|_| CommerceError::LockError)?;

        Ok(levels
            .iter()
            .filter(|(k, _)| &k.product_id == product_id)
            .map(|(_, v)| v.clone())
            .collect())
    }

    // ========================================================================
    // STOCK OPERATIONS
    // ========================================================================

    /// Reserves stock for an order.
    pub fn reserve_stock(
        &self, product_id: &ProductId, location_id: &LocationId, quantity: u32,
        reference: impl Into<String>,
    ) -> Result<(), CommerceError> {
        let key = InventoryKey {
            product_id:  product_id.clone(),
            variant_id:  None,
            location_id: location_id.clone(),
        };

        let mut levels = self.levels.lock().map_err(|_| CommerceError::LockError)?;

        let level = levels
            .get_mut(&key)
            .ok_or_else(|| CommerceError::InventoryNotFound(product_id.0.to_string()))?;

        if level.available < i64::from(quantity) {
            return Err(CommerceError::InsufficientInventory {
                product_id: product_id.0.to_string(),
                available:  level.available.max(0) as u32,
                requested:  quantity,
            });
        }

        let previous = level.committed;
        level.committed = level.committed.saturating_add(i64::from(quantity));
        level.recalculate_available();

        // Clone product_id and location_id for the adjustment since we still need
        // references for error handling
        let adjustment = InventoryAdjustment::new(
            product_id.clone(),
            location_id.clone(),
            AdjustmentType::Reserved,
            i64::from(quantity),
            previous,
            "Stock reserved for order",
        )
        .with_reference(reference);

        drop(levels);
        self.record_adjustment(adjustment)?;

        Ok(())
    }

    /// Releases reserved stock (e.g., order cancelled).
    pub fn release_stock(
        &self, product_id: &ProductId, location_id: &LocationId, quantity: u32,
        reference: impl Into<String>,
    ) -> Result<(), CommerceError> {
        let key = InventoryKey {
            product_id:  product_id.clone(),
            variant_id:  None,
            location_id: location_id.clone(),
        };

        let mut levels = self.levels.lock().map_err(|_| CommerceError::LockError)?;

        let level = levels
            .get_mut(&key)
            .ok_or_else(|| CommerceError::InventoryNotFound(product_id.0.to_string()))?;

        let previous = level.committed;
        level.committed = level.committed.saturating_sub(i64::from(quantity));
        level.recalculate_available();

        let adjustment = InventoryAdjustment::new(
            product_id.clone(),
            location_id.clone(),
            AdjustmentType::Unreserved,
            -(i64::from(quantity)),
            previous,
            "Stock released",
        )
        .with_reference(reference);

        drop(levels);
        self.record_adjustment(adjustment)?;

        Ok(())
    }

    /// Commits stock (deduct from on-hand for shipped order).
    pub fn commit_stock(
        &self, product_id: &ProductId, location_id: &LocationId, quantity: u32,
        reference: impl Into<String>,
    ) -> Result<(), CommerceError> {
        let key = InventoryKey {
            product_id:  product_id.clone(),
            variant_id:  None,
            location_id: location_id.clone(),
        };

        let mut levels = self.levels.lock().map_err(|_| CommerceError::LockError)?;

        let level = levels
            .get_mut(&key)
            .ok_or_else(|| CommerceError::InventoryNotFound(product_id.0.to_string()))?;

        let previous = level.on_hand;
        level.on_hand = level.on_hand.saturating_sub(i64::from(quantity));
        level.committed = level.committed.saturating_sub(i64::from(quantity));
        level.recalculate_available();

        let adjustment = InventoryAdjustment::new(
            product_id.clone(),
            location_id.clone(),
            AdjustmentType::Shipped,
            -(i64::from(quantity)),
            previous,
            "Stock shipped",
        )
        .with_reference(reference);

        drop(levels);
        self.record_adjustment(adjustment)?;

        Ok(())
    }

    /// Receives stock (add to on-hand).
    pub fn receive_stock(
        &self, product_id: &ProductId, location_id: &LocationId, quantity: u32,
        reference: impl Into<String>,
    ) -> Result<(), CommerceError> {
        // Clone for key - required since we need owned values in the key
        let product_id_owned = product_id.clone();
        let location_id_owned = location_id.clone();

        let key = InventoryKey {
            product_id:  product_id_owned.clone(),
            variant_id:  None,
            location_id: location_id_owned.clone(),
        };

        let mut levels = self.levels.lock().map_err(|_| CommerceError::LockError)?;

        let level = levels.entry(key).or_insert_with(|| {
            InventoryLevel::new(product_id_owned.clone(), location_id_owned.clone())
        });

        let previous = level.on_hand;
        level.on_hand = level.on_hand.saturating_add(i64::from(quantity));
        level.recalculate_available();

        let adjustment = InventoryAdjustment::new(
            product_id.clone(),
            location_id.clone(),
            AdjustmentType::Received,
            i64::from(quantity),
            previous,
            "Stock received",
        )
        .with_reference(reference);

        drop(levels);
        self.record_adjustment(adjustment)?;

        Ok(())
    }

    // ========================================================================
    // TRANSFER OPERATIONS
    // ========================================================================

    /// Creates a stock transfer.
    pub fn create_transfer(
        &self, from_location: LocationId, to_location: LocationId,
    ) -> Result<StockTransfer, CommerceError> {
        // Validate locations exist
        let _ = self.get_location(&from_location)?;
        let _ = self.get_location(&to_location)?;

        let transfer = StockTransfer::new(from_location, to_location);
        let transfer_id = transfer.id.to_string();

        let mut transfers = self.transfers.lock().map_err(|_| CommerceError::LockError)?;
        transfers.insert(transfer_id, transfer.clone());

        Ok(transfer)
    }

    /// Gets a transfer.
    pub fn get_transfer(&self, id: &str) -> Result<StockTransfer, CommerceError> {
        let transfers = self.transfers.lock().map_err(|_| CommerceError::LockError)?;
        transfers
            .get(id)
            .cloned()
            .ok_or_else(|| CommerceError::TransferNotFound(id.to_string()))
    }

    /// Completes a transfer.
    pub fn complete_transfer(&self, transfer_id: &str) -> Result<(), CommerceError> {
        // First, get the transfer data and validate status
        let (items, from_location, to_location) = {
            let transfers = self.transfers.lock().map_err(|_| CommerceError::LockError)?;

            let transfer = transfers
                .get(transfer_id)
                .ok_or_else(|| CommerceError::TransferNotFound(transfer_id.to_string()))?;

            if transfer.status != TransferStatus::Pending
                && transfer.status != TransferStatus::InProgress
            {
                return Err(CommerceError::InvalidTransferStatus);
            }

            // Clone the data we need
            (
                transfer.items.clone(),
                transfer.from_location.clone(),
                transfer.to_location.clone(),
            )
        };

        // Move stock for each item (lock is released)
        for item in &items {
            let reference = format!("Transfer {}", transfer_id);

            // Deduct from source
            self.commit_stock(&item.product_id, &from_location, item.quantity, &reference)?;

            // Add to destination
            self.receive_stock(&item.product_id, &to_location, item.quantity, &reference)?;
        }

        // Update transfer status
        let mut transfers = self.transfers.lock().map_err(|_| CommerceError::LockError)?;
        let transfer = transfers
            .get_mut(transfer_id)
            .ok_or_else(|| CommerceError::TransferNotFound(transfer_id.to_string()))?;

        transfer.status = TransferStatus::Completed;
        transfer.arrived_at = Some(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        );

        Ok(())
    }

    // ========================================================================
    // SYNC OPERATIONS
    // ========================================================================

    /// Registers an external inventory source.
    pub fn register_source(&self, source: ExternalInventorySource) -> Result<(), CommerceError> {
        let mut sources = self.sources.lock().map_err(|_| CommerceError::LockError)?;
        sources.insert(source.id.to_string(), source);
        Ok(())
    }

    /// Applies inventory changes from external source.
    pub fn apply_sync_changes(
        &self, source_id: &str, changes: Vec<InventoryChange>,
    ) -> Result<SyncResult, CommerceError> {
        let start = Instant::now();
        let mut processed = 0u32;
        let mut updated = 0u32;
        let mut failed = 0u32;
        let mut errors = Vec::new();

        for change in changes {
            processed += 1;

            // Attempt to apply change
            let result = self.apply_single_change(&change, source_id);
            match result {
                Ok(()) => updated += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(format!("Product {}: {}", change.product_id, e));
                },
            }
        }

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let status = if failed == 0 {
            SyncStatus::Success
        } else if updated > 0 {
            SyncStatus::Partial
        } else {
            SyncStatus::Failed
        };

        Ok(SyncResult {
            source_id: source_id.to_string(),
            status,
            items_processed: processed,
            items_updated: updated,
            items_failed: failed,
            errors,
            synced_at: now,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Applies a single inventory change.
    fn apply_single_change(
        &self, change: &InventoryChange, source_id: &str,
    ) -> Result<(), CommerceError> {
        let product_id = ProductId::new(&change.product_id);
        let location_id = LocationId::new(&change.location_id);

        // Clone once for key, reuse for or_insert_with
        let key = InventoryKey {
            product_id:  product_id.clone(),
            variant_id:  None,
            location_id: location_id.clone(),
        };

        let mut levels = self.levels.lock().map_err(|_| CommerceError::LockError)?;

        // Use key's cloned values for or_insert_with to avoid additional clones
        let key_product_id = product_id.clone();
        let key_location_id = location_id.clone();
        let level = levels
            .entry(key)
            .or_insert_with(|| InventoryLevel::new(key_product_id, key_location_id));

        match change.change_type {
            InventoryChangeType::Set => {
                level.on_hand = change.quantity;
            },
            InventoryChangeType::Increment => {
                level.on_hand = level.on_hand.saturating_add(change.quantity);
            },
            InventoryChangeType::Decrement => {
                level.on_hand = level.on_hand.saturating_sub(change.quantity);
            },
        }

        level.recalculate_available();

        // Record the sync adjustment
        let adjustment = InventoryAdjustment::new(
            product_id,
            location_id,
            AdjustmentType::Adjustment,
            change.quantity,
            level.on_hand - change.quantity,
            format!("Sync from {}", source_id),
        );

        drop(levels);
        self.record_adjustment(adjustment)?;

        Ok(())
    }

    // ========================================================================
    // LOW STOCK & REORDER
    // ========================================================================

    /// Gets products with low stock.
    pub fn get_low_stock_products(&self) -> Result<Vec<InventoryLevel>, CommerceError> {
        let levels = self.levels.lock().map_err(|_| CommerceError::LockError)?;

        Ok(levels.values().filter(|l| l.is_low_stock()).cloned().collect())
    }

    /// Gets products needing reorder.
    pub fn get_reorder_needed(&self) -> Result<Vec<InventoryLevel>, CommerceError> {
        let levels = self.levels.lock().map_err(|_| CommerceError::LockError)?;

        Ok(levels.values().filter(|l| l.needs_reorder()).cloned().collect())
    }

    /// Gets out-of-stock products.
    pub fn get_out_of_stock(&self) -> Result<Vec<InventoryLevel>, CommerceError> {
        let levels = self.levels.lock().map_err(|_| CommerceError::LockError)?;

        Ok(levels.values().filter(|l| l.is_out_of_stock()).cloned().collect())
    }

    // ========================================================================
    // ADJUSTMENT HISTORY
    // ========================================================================

    /// Records an adjustment.
    fn record_adjustment(&self, adjustment: InventoryAdjustment) -> Result<(), CommerceError> {
        let mut adjustments = self.adjustments.lock().map_err(|_| CommerceError::LockError)?;
        adjustments.push(adjustment);
        Ok(())
    }

    /// Gets adjustment history for a product.
    pub fn get_adjustment_history(
        &self, product_id: &ProductId, limit: Option<usize>,
    ) -> Result<Vec<InventoryAdjustment>, CommerceError> {
        let adjustments = self.adjustments.lock().map_err(|_| CommerceError::LockError)?;

        let mut history: Vec<_> =
            adjustments.iter().filter(|a| &a.product_id == product_id).cloned().collect();

        // Sort by most recent first
        history.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        if let Some(limit) = limit {
            history.truncate(limit);
        }

        Ok(history)
    }
}

impl Default for InventoryService {
    fn default() -> Self {
        Self::new()
    }
}
