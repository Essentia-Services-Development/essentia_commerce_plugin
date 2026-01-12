//! Error types for the Commerce plugin

use std::fmt;

/// Commerce-specific errors.
#[derive(Debug, Clone)]
pub enum CommerceError {
    /// Lock acquisition failed.
    LockError,
    /// Product not found.
    ProductNotFound(String),
    /// Product already exists.
    ProductAlreadyExists(String),
    /// SKU already exists.
    SkuAlreadyExists(String),
    /// Category not found.
    CategoryNotFound(String),
    /// Category already exists.
    CategoryAlreadyExists(String),
    /// Cart not found.
    CartNotFound(String),
    /// Cart is empty.
    CartEmpty,
    /// Cart is not active.
    CartNotActive,
    /// Cart has expired.
    CartExpired,
    /// Item not in cart.
    ItemNotInCart(String),
    /// Invalid quantity.
    InvalidQuantity,
    /// Product not available for purchase.
    ProductNotAvailable(String),
    /// Insufficient inventory.
    InsufficientInventory {
        /// Product ID.
        product_id: String,
        /// Available quantity.
        available: u32,
        /// Requested quantity.
        requested: u32,
    },
    /// Currency mismatch.
    CurrencyMismatch {
        /// Expected currency.
        expected: String,
        /// Received currency.
        got: String,
    },
    /// Discount already applied.
    DiscountAlreadyApplied(String),
    /// Discount not found.
    DiscountNotFound(String),
    /// Shipping address required.
    ShippingAddressRequired,
    /// Order not found.
    OrderNotFound(String),
    /// Order cannot be cancelled.
    OrderNotCancellable(String),
    /// Location not found.
    LocationNotFound(String),
    /// Location already exists.
    LocationAlreadyExists(String),
    /// Inventory record not found.
    InventoryNotFound(String),
    /// Transfer not found.
    TransferNotFound(String),
    /// Invalid transfer status.
    InvalidTransferStatus,
    /// Validation error.
    ValidationError(String),
    /// Internal error.
    InternalError(String),
}

impl fmt::Display for CommerceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LockError => write!(f, "Failed to acquire lock"),
            Self::ProductNotFound(id) => write!(f, "Product not found: {}", id),
            Self::ProductAlreadyExists(id) => write!(f, "Product already exists: {}", id),
            Self::SkuAlreadyExists(sku) => write!(f, "SKU already exists: {}", sku),
            Self::CategoryNotFound(id) => write!(f, "Category not found: {}", id),
            Self::CategoryAlreadyExists(id) => write!(f, "Category already exists: {}", id),
            Self::CartNotFound(id) => write!(f, "Cart not found: {}", id),
            Self::CartEmpty => write!(f, "Cart is empty"),
            Self::CartNotActive => write!(f, "Cart is not active"),
            Self::CartExpired => write!(f, "Cart has expired"),
            Self::ItemNotInCart(id) => write!(f, "Item not in cart: {}", id),
            Self::InvalidQuantity => write!(f, "Invalid quantity"),
            Self::ProductNotAvailable(id) => write!(f, "Product not available: {}", id),
            Self::InsufficientInventory { product_id, available, requested } => {
                write!(
                    f,
                    "Insufficient inventory for {}: available {}, requested {}",
                    product_id, available, requested
                )
            }
            Self::CurrencyMismatch { expected, got } => {
                write!(f, "Currency mismatch: expected {}, got {}", expected, got)
            }
            Self::DiscountAlreadyApplied(code) => write!(f, "Discount already applied: {}", code),
            Self::DiscountNotFound(code) => write!(f, "Discount not found: {}", code),
            Self::ShippingAddressRequired => write!(f, "Shipping address required"),
            Self::OrderNotFound(id) => write!(f, "Order not found: {}", id),
            Self::OrderNotCancellable(id) => write!(f, "Order cannot be cancelled: {}", id),
            Self::LocationNotFound(id) => write!(f, "Location not found: {}", id),
            Self::LocationAlreadyExists(id) => write!(f, "Location already exists: {}", id),
            Self::InventoryNotFound(id) => write!(f, "Inventory record not found: {}", id),
            Self::TransferNotFound(id) => write!(f, "Transfer not found: {}", id),
            Self::InvalidTransferStatus => write!(f, "Invalid transfer status"),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for CommerceError {}

impl From<CommerceError> for essentia_api::PluginError {
    fn from(err: CommerceError) -> Self {
        essentia_api::PluginError::ExecutionFailed(err.to_string())
    }
}

/// Result type for commerce operations.
pub type CommerceResult<T> = Result<T, CommerceError>;
