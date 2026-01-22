//! # Inventory Sync Types (GAP-220-D-004)
//!
//! Type definitions for real-time inventory synchronization and management.

use crate::types::product_catalog::{ProductId, Sku};

// ============================================================================
// CORE TYPES
// ============================================================================

/// Inventory location identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocationId(pub String);

impl std::fmt::Display for LocationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

impl std::fmt::Display for InventoryLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "InventoryLevel {{ product: {}, location: {}, available: {}, on_hand: {}, committed: \
             {} }}",
            self.product_id, self.location_id, self.available, self.on_hand, self.committed
        )
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

impl std::fmt::Display for StockTransfer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "StockTransfer {{ id: {}, from: {}, to: {}, status: {:?}, items: {} }}",
            self.id,
            self.from_location,
            self.to_location,
            self.status,
            self.items.len()
        )
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

/// Inventory management service.
#[derive(Debug)]
pub struct InventoryService {
    /// Inventory levels.
    pub levels:
        std::sync::Arc<std::sync::Mutex<std::collections::HashMap<InventoryKey, InventoryLevel>>>,
    /// Locations.
    pub locations:
        std::sync::Arc<std::sync::Mutex<std::collections::HashMap<LocationId, InventoryLocation>>>,
    /// Adjustment history.
    pub adjustments: std::sync::Arc<std::sync::Mutex<Vec<InventoryAdjustment>>>,
    /// Pending transfers.
    pub transfers:
        std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, StockTransfer>>>,
    /// External sources.
    pub sources: std::sync::Arc<
        std::sync::Mutex<std::collections::HashMap<String, ExternalInventorySource>>,
    >,
}

/// Key for inventory level lookup.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InventoryKey {
    /// Product ID.
    pub product_id:  ProductId,
    /// Variant ID.
    pub variant_id:  Option<ProductId>,
    /// Location ID.
    pub location_id: LocationId,
}
