//! # Inventory Sync (GAP-220-D-004)
//!
//! Real-time inventory synchronization and management for the e-commerce
//! platform.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    errors::CommerceError,
    r#impl::product_catalog::{ProductId, Sku},
};

// ============================================================================
// CORE TYPES
// ============================================================================

/// Inventory location identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocationId(pub String);

impl LocationId {
    /// Creates a new location ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Default warehouse location.
    #[must_use]
    pub fn default_warehouse() -> Self {
        Self("warehouse-main".to_string())
    }
}

/// Warehouse/location definition.
#[derive(Debug, Clone)]
pub struct InventoryLocation {
    /// Location ID.
    pub id:                   LocationId,
    /// Location name.
    pub name:                 String,
    /// Location type.
    pub location_type:        LocationType,
    /// Street address.
    pub address:              String,
    /// City.
    pub city:                 String,
    /// State/province.
    pub state:                String,
    /// Country code.
    pub country_code:         String,
    /// Postal code.
    pub postal_code:          String,
    /// Whether location is active.
    pub is_active:            bool,
    /// Priority for fulfillment (lower = higher priority).
    pub fulfillment_priority: u32,
    /// Whether location can ship orders.
    pub can_ship:             bool,
    /// Whether location allows in-store pickup.
    pub allows_pickup:        bool,
}

impl InventoryLocation {
    /// Creates a new warehouse location.
    #[must_use]
    pub fn warehouse(id: LocationId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            location_type: LocationType::Warehouse,
            address: String::new(),
            city: String::new(),
            state: String::new(),
            country_code: String::new(),
            postal_code: String::new(),
            is_active: true,
            fulfillment_priority: 1,
            can_ship: true,
            allows_pickup: false,
        }
    }

    /// Creates a store location.
    #[must_use]
    pub fn store(id: LocationId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            location_type: LocationType::Store,
            address: String::new(),
            city: String::new(),
            state: String::new(),
            country_code: String::new(),
            postal_code: String::new(),
            is_active: true,
            fulfillment_priority: 10,
            can_ship: true,
            allows_pickup: true,
        }
    }
}

/// Location type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationType {
    /// Main warehouse.
    Warehouse,
    /// Distribution center.
    DistributionCenter,
    /// Retail store.
    Store,
    /// Drop-ship supplier.
    Dropship,
    /// Virtual (digital products).
    Virtual,
}

/// Inventory level for a product at a location.
#[derive(Debug, Clone)]
pub struct InventoryLevel {
    /// Product ID.
    pub product_id:          ProductId,
    /// Variant ID (if applicable).
    pub variant_id:          Option<ProductId>,
    /// Location ID.
    pub location_id:         LocationId,
    /// Available quantity (can be sold).
    pub available:           i64,
    /// Committed quantity (reserved for orders).
    pub committed:           i64,
    /// On-hand quantity (physically in stock).
    pub on_hand:             i64,
    /// Incoming quantity (on order from supplier).
    pub incoming:            i64,
    /// Damaged/unsellable quantity.
    pub damaged:             i64,
    /// Low stock threshold.
    pub low_stock_threshold: u32,
    /// Reorder point.
    pub reorder_point:       u32,
    /// Reorder quantity.
    pub reorder_quantity:    u32,
    /// Safety stock level.
    pub safety_stock:        u32,
    /// Last stock count date.
    pub last_count_at:       Option<u64>,
    /// Last update timestamp.
    pub updated_at:          u64,
}

impl InventoryLevel {
    /// Creates a new inventory level.
    #[must_use]
    pub fn new(product_id: ProductId, location_id: LocationId) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            product_id,
            variant_id: None,
            location_id,
            available: 0,
            committed: 0,
            on_hand: 0,
            incoming: 0,
            damaged: 0,
            low_stock_threshold: 10,
            reorder_point: 20,
            reorder_quantity: 50,
            safety_stock: 5,
            last_count_at: None,
            updated_at: now,
        }
    }

    /// Whether stock is low.
    #[must_use]
    pub fn is_low_stock(&self) -> bool {
        self.available > 0 && self.available <= i64::from(self.low_stock_threshold)
    }

    /// Whether stock is out.
    #[must_use]
    pub fn is_out_of_stock(&self) -> bool {
        self.available <= 0
    }

    /// Whether reorder is needed.
    #[must_use]
    pub fn needs_reorder(&self) -> bool {
        self.available <= i64::from(self.reorder_point)
    }

    /// Recalculates available quantity.
    pub fn recalculate_available(&mut self) {
        self.available = self.on_hand.saturating_sub(self.committed).saturating_sub(self.damaged);
        self.touch();
    }

    /// Updates timestamp.
    fn touch(&mut self) {
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }
}

// ============================================================================
// INVENTORY ADJUSTMENT
// ============================================================================

/// Type of inventory adjustment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjustmentType {
    /// Stock received from supplier.
    Received,
    /// Stock shipped to customer.
    Shipped,
    /// Stock returned by customer.
    Returned,
    /// Stock counted/corrected.
    Adjustment,
    /// Stock transferred between locations.
    Transfer,
    /// Stock reserved for order.
    Reserved,
    /// Stock unreserved (order cancelled).
    Unreserved,
    /// Stock damaged.
    Damaged,
    /// Stock scrapped/disposed.
    Scrapped,
    /// Cycle count adjustment.
    CycleCount,
}

/// Inventory adjustment record.
#[derive(Debug, Clone)]
pub struct InventoryAdjustment {
    /// Adjustment ID.
    pub id:                String,
    /// Product ID.
    pub product_id:        ProductId,
    /// Variant ID.
    pub variant_id:        Option<ProductId>,
    /// Location ID.
    pub location_id:       LocationId,
    /// Adjustment type.
    pub adjustment_type:   AdjustmentType,
    /// Quantity adjusted (positive or negative).
    pub quantity:          i64,
    /// Previous on-hand quantity.
    pub previous_quantity: i64,
    /// New on-hand quantity.
    pub new_quantity:      i64,
    /// Reference (order ID, PO number, etc).
    pub reference:         Option<String>,
    /// Reason for adjustment.
    pub reason:            String,
    /// User who made adjustment.
    pub user:              Option<String>,
    /// Adjustment timestamp.
    pub created_at:        u64,
}

impl InventoryAdjustment {
    /// Creates a new adjustment record.
    #[must_use]
    pub fn new(
        product_id: ProductId, location_id: LocationId, adjustment_type: AdjustmentType,
        quantity: i64, previous_quantity: i64, reason: impl Into<String>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            id: format!("adj-{}", now),
            product_id,
            variant_id: None,
            location_id,
            adjustment_type,
            quantity,
            previous_quantity,
            new_quantity: previous_quantity + quantity,
            reference: None,
            reason: reason.into(),
            user: None,
            created_at: now,
        }
    }

    /// Sets reference.
    #[must_use]
    pub fn with_reference(mut self, reference: impl Into<String>) -> Self {
        self.reference = Some(reference.into());
        self
    }

    /// Sets user.
    #[must_use]
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }
}

// ============================================================================
// STOCK TRANSFER
// ============================================================================

/// Stock transfer between locations.
#[derive(Debug, Clone)]
pub struct StockTransfer {
    /// Transfer ID.
    pub id:               String,
    /// Source location.
    pub from_location:    LocationId,
    /// Destination location.
    pub to_location:      LocationId,
    /// Transfer status.
    pub status:           TransferStatus,
    /// Items being transferred.
    pub items:            Vec<TransferItem>,
    /// Expected arrival date.
    pub expected_arrival: Option<u64>,
    /// Actual arrival date.
    pub arrived_at:       Option<u64>,
    /// Notes.
    pub notes:            Option<String>,
    /// Creation timestamp.
    pub created_at:       u64,
    /// Last update timestamp.
    pub updated_at:       u64,
}

/// Transfer status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransferStatus {
    /// Transfer pending.
    #[default]
    Pending,
    /// Transfer in progress.
    InProgress,
    /// Transfer completed.
    Completed,
    /// Transfer cancelled.
    Cancelled,
}

/// Item in a transfer.
#[derive(Debug, Clone)]
pub struct TransferItem {
    /// Product ID.
    pub product_id:        ProductId,
    /// Variant ID.
    pub variant_id:        Option<ProductId>,
    /// Quantity to transfer.
    pub quantity:          u32,
    /// Quantity received.
    pub quantity_received: u32,
}

impl StockTransfer {
    /// Creates a new transfer.
    #[must_use]
    pub fn new(from_location: LocationId, to_location: LocationId) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            id: format!("transfer-{}", now),
            from_location,
            to_location,
            status: TransferStatus::Pending,
            items: Vec::new(),
            expected_arrival: None,
            arrived_at: None,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Adds an item to the transfer.
    pub fn add_item(&mut self, product_id: ProductId, quantity: u32) {
        self.items.push(TransferItem {
            product_id,
            variant_id: None,
            quantity,
            quantity_received: 0,
        });
        self.touch();
    }

    /// Updates timestamp.
    fn touch(&mut self) {
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }
}

// ============================================================================
// SYNC OPERATIONS
// ============================================================================

/// External inventory source for synchronization.
#[derive(Debug, Clone)]
pub struct ExternalInventorySource {
    /// Source ID.
    pub id:                 String,
    /// Source name.
    pub name:               String,
    /// Source type.
    pub source_type:        ExternalSourceType,
    /// API endpoint URL.
    pub endpoint_url:       Option<String>,
    /// Whether sync is enabled.
    pub sync_enabled:       bool,
    /// Sync interval in seconds.
    pub sync_interval_secs: u64,
    /// Last successful sync.
    pub last_sync_at:       Option<u64>,
    /// Last sync status.
    pub last_sync_status:   Option<SyncStatus>,
}

/// External source type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalSourceType {
    /// ERP system.
    Erp,
    /// Warehouse management system.
    Wms,
    /// Point of sale.
    Pos,
    /// Marketplace (Amazon, eBay, etc).
    Marketplace,
    /// Supplier/dropship.
    Supplier,
    /// Manual import.
    Manual,
}

/// Sync status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    /// Sync successful.
    Success,
    /// Sync failed.
    Failed,
    /// Sync in progress.
    InProgress,
    /// Sync partially successful.
    Partial,
}

/// Sync result.
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Source ID.
    pub source_id:       String,
    /// Status.
    pub status:          SyncStatus,
    /// Items processed.
    pub items_processed: u32,
    /// Items updated.
    pub items_updated:   u32,
    /// Items failed.
    pub items_failed:    u32,
    /// Error messages.
    pub errors:          Vec<String>,
    /// Sync timestamp.
    pub synced_at:       u64,
    /// Duration in milliseconds.
    pub duration_ms:     u64,
}

/// Inventory change for sync.
#[derive(Debug, Clone)]
pub struct InventoryChange {
    /// Product ID (or external ID).
    pub product_id:       String,
    /// SKU.
    pub sku:              Option<Sku>,
    /// Location ID (or external ID).
    pub location_id:      String,
    /// New quantity.
    pub quantity:         i64,
    /// Change type.
    pub change_type:      InventoryChangeType,
    /// Timestamp of change at source.
    pub source_timestamp: Option<u64>,
}

/// Type of inventory change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryChangeType {
    /// Set absolute quantity.
    Set,
    /// Increment quantity.
    Increment,
    /// Decrement quantity.
    Decrement,
}

// ============================================================================
// INVENTORY SERVICE
// ============================================================================

/// Key for inventory level lookup.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct InventoryKey {
    product_id:  ProductId,
    variant_id:  Option<ProductId>,
    location_id: LocationId,
}

/// Inventory management service.
#[derive(Debug)]
pub struct InventoryService {
    /// Inventory levels.
    levels:      Arc<Mutex<HashMap<InventoryKey, InventoryLevel>>>,
    /// Locations.
    locations:   Arc<Mutex<HashMap<LocationId, InventoryLocation>>>,
    /// Adjustment history.
    adjustments: Arc<Mutex<Vec<InventoryAdjustment>>>,
    /// Pending transfers.
    transfers:   Arc<Mutex<HashMap<String, StockTransfer>>>,
    /// External sources.
    sources:     Arc<Mutex<HashMap<String, ExternalInventorySource>>>,
}

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
            return Err(CommerceError::LocationAlreadyExists(location.id.0.clone()));
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
            .ok_or_else(|| CommerceError::LocationNotFound(id.0.clone()))
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

        // Record adjustment
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
            .ok_or_else(|| CommerceError::InventoryNotFound(product_id.0.clone()))
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
            .ok_or_else(|| CommerceError::InventoryNotFound(product_id.0.clone()))?;

        if level.available < i64::from(quantity) {
            return Err(CommerceError::InsufficientInventory {
                product_id: product_id.0.clone(),
                available:  level.available.max(0) as u32,
                requested:  quantity,
            });
        }

        let previous = level.committed;
        level.committed = level.committed.saturating_add(i64::from(quantity));
        level.recalculate_available();

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
            .ok_or_else(|| CommerceError::InventoryNotFound(product_id.0.clone()))?;

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
            .ok_or_else(|| CommerceError::InventoryNotFound(product_id.0.clone()))?;

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
        let key = InventoryKey {
            product_id:  product_id.clone(),
            variant_id:  None,
            location_id: location_id.clone(),
        };

        let mut levels = self.levels.lock().map_err(|_| CommerceError::LockError)?;

        let level = levels
            .entry(key)
            .or_insert_with(|| InventoryLevel::new(product_id.clone(), location_id.clone()));

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
        let transfer_id = transfer.id.clone();

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
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
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
        sources.insert(source.id.clone(), source);
        Ok(())
    }

    /// Applies inventory changes from external source.
    pub fn apply_sync_changes(
        &self, source_id: &str, changes: Vec<InventoryChange>,
    ) -> Result<SyncResult, CommerceError> {
        let start = std::time::Instant::now();
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

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
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

        let key = InventoryKey {
            product_id:  product_id.clone(),
            variant_id:  None,
            location_id: location_id.clone(),
        };

        let mut levels = self.levels.lock().map_err(|_| CommerceError::LockError)?;

        let level = levels
            .entry(key)
            .or_insert_with(|| InventoryLevel::new(product_id.clone(), location_id.clone()));

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

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_service_creation() {
        let service = InventoryService::new();

        // Default warehouse should exist
        let location = service.get_location(&LocationId::default_warehouse());
        assert!(location.is_ok());
    }

    #[test]
    fn test_set_and_get_inventory() {
        let service = InventoryService::new();
        let product_id = ProductId::new("prod-001");
        let location_id = LocationId::default_warehouse();

        service
            .set_inventory(
                product_id.clone(),
                location_id.clone(),
                100,
                "Initial stock",
            )
            .expect("set inventory");

        let level = service.get_inventory(&product_id, &location_id).expect("get");

        assert_eq!(level.on_hand, 100);
        assert_eq!(level.available, 100);
    }

    #[test]
    fn test_reserve_stock() {
        let service = InventoryService::new();
        let product_id = ProductId::new("prod-001");
        let location_id = LocationId::default_warehouse();

        service
            .set_inventory(product_id.clone(), location_id.clone(), 100, "Initial")
            .expect("set");

        service
            .reserve_stock(&product_id, &location_id, 30, "ORD-001")
            .expect("reserve");

        let level = service.get_inventory(&product_id, &location_id).expect("get");
        assert_eq!(level.on_hand, 100);
        assert_eq!(level.committed, 30);
        assert_eq!(level.available, 70);
    }

    #[test]
    fn test_reserve_insufficient_stock() {
        let service = InventoryService::new();
        let product_id = ProductId::new("prod-001");
        let location_id = LocationId::default_warehouse();

        service
            .set_inventory(product_id.clone(), location_id.clone(), 10, "Low stock")
            .expect("set");

        let result = service.reserve_stock(&product_id, &location_id, 50, "ORD-001");
        assert!(result.is_err());
    }

    #[test]
    fn test_commit_stock() {
        let service = InventoryService::new();
        let product_id = ProductId::new("prod-001");
        let location_id = LocationId::default_warehouse();

        service
            .set_inventory(product_id.clone(), location_id.clone(), 100, "Initial")
            .expect("set");

        service
            .reserve_stock(&product_id, &location_id, 30, "ORD-001")
            .expect("reserve");
        service.commit_stock(&product_id, &location_id, 30, "ORD-001").expect("commit");

        let level = service.get_inventory(&product_id, &location_id).expect("get");
        assert_eq!(level.on_hand, 70);
        assert_eq!(level.committed, 0);
        assert_eq!(level.available, 70);
    }

    #[test]
    fn test_receive_stock() {
        let service = InventoryService::new();
        let product_id = ProductId::new("prod-001");
        let location_id = LocationId::default_warehouse();

        service
            .set_inventory(product_id.clone(), location_id.clone(), 50, "Initial")
            .expect("set");

        service
            .receive_stock(&product_id, &location_id, 100, "PO-001")
            .expect("receive");

        let level = service.get_inventory(&product_id, &location_id).expect("get");
        assert_eq!(level.on_hand, 150);
        assert_eq!(level.available, 150);
    }

    #[test]
    fn test_low_stock_detection() {
        let service = InventoryService::new();
        let product_id = ProductId::new("prod-001");
        let location_id = LocationId::default_warehouse();

        service
            .set_inventory(product_id.clone(), location_id.clone(), 5, "Low stock")
            .expect("set");

        let level = service.get_inventory(&product_id, &location_id).expect("get");
        assert!(level.is_low_stock());

        let low_stock = service.get_low_stock_products().expect("get low");
        assert_eq!(low_stock.len(), 1);
    }

    #[test]
    fn test_total_available_across_locations() {
        let service = InventoryService::new();
        let product_id = ProductId::new("prod-001");

        let location1 = LocationId::default_warehouse();
        let location2 = LocationId::new("warehouse-secondary");

        service
            .add_location(InventoryLocation::warehouse(
                location2.clone(),
                "Secondary Warehouse",
            ))
            .expect("add location");

        service
            .set_inventory(product_id.clone(), location1, 100, "Stock 1")
            .expect("set 1");
        service
            .set_inventory(product_id.clone(), location2, 50, "Stock 2")
            .expect("set 2");

        let total = service.get_total_available(&product_id).expect("total");
        assert_eq!(total, 150);
    }

    #[test]
    fn test_adjustment_history() {
        let service = InventoryService::new();
        let product_id = ProductId::new("prod-001");
        let location_id = LocationId::default_warehouse();

        service
            .set_inventory(product_id.clone(), location_id.clone(), 100, "Initial")
            .expect("set");
        service.receive_stock(&product_id, &location_id, 50, "PO-001").expect("receive");
        service
            .reserve_stock(&product_id, &location_id, 30, "ORD-001")
            .expect("reserve");

        let history = service.get_adjustment_history(&product_id, None).expect("history");
        assert_eq!(history.len(), 3);
    }
}
