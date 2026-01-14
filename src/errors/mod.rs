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
        available:  u32,
        /// Requested quantity.
        requested:  u32,
    },
    /// Currency mismatch.
    CurrencyMismatch {
        /// Expected currency.
        expected: String,
        /// Received currency.
        got:      String,
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
    /// Payment plugin not configured.
    PaymentPluginNotConfigured,
    /// Payment error.
    PaymentError(String),
    /// Payment failed.
    PaymentFailed(String),
    /// Blockchain plugin not configured.
    BlockchainPluginNotConfigured,
    /// Blockchain error.
    BlockchainError(String),
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
            },
            Self::CurrencyMismatch { expected, got } => {
                write!(f, "Currency mismatch: expected {}, got {}", expected, got)
            },
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
            Self::PaymentPluginNotConfigured => write!(f, "Payment plugin not configured"),
            Self::PaymentError(msg) => write!(f, "Payment error: {}", msg),
            Self::PaymentFailed(msg) => write!(f, "Payment failed: {}", msg),
            Self::BlockchainPluginNotConfigured => write!(f, "Blockchain plugin not configured"),
            Self::BlockchainError(msg) => write!(f, "Blockchain error: {}", msg),
        }
    }
}

impl std::error::Error for CommerceError {}

impl From<CommerceError> for essentia_api::PluginError {
    fn from(err: CommerceError) -> Self {
        essentia_api::PluginError::ExecutionFailed(err.to_string())
    }
}

/// Marketplace-specific errors.
#[derive(Debug, Clone)]
pub enum MarketplaceError {
    /// Listing not found
    ListingNotFound,
    /// Listing not active
    ListingNotActive,
    /// Seller not found
    SellerNotFound,
    /// Invalid listing data
    InvalidListing,
    /// Payment amount required
    AmountRequired,
    /// Payment amount below minimum
    BelowMinimum,
    /// Order not found
    OrderNotFound,
    /// Insufficient funds
    InsufficientFunds,
    /// Payment failed
    PaymentFailed,
    /// Escrow error
    EscrowError(String),
    /// Search error
    SearchError(String),
    /// Serialization error
    SerializationError(String),
    /// IO error
    IoError(String),
    /// Invalid access token
    InvalidToken,
    /// Token expired
    TokenExpired,
    /// Download limit reached
    DownloadLimitReached,
    /// No content providers available
    NoProviders,
    /// Content not found
    ContentNotFound,
    /// Insufficient funds for escrow
    InsufficientFundsForEscrow,
    /// Escrow already exists
    EscrowExists,
    /// Escrow not found
    EscrowNotFound,
    /// Invalid escrow state for operation
    InvalidEscrowState,
    /// Release conditions not met
    ReleaseConditionsNotMet,
}

impl fmt::Display for MarketplaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ListingNotFound => write!(f, "Listing not found"),
            Self::ListingNotActive => write!(f, "Listing not active"),
            Self::SellerNotFound => write!(f, "Seller not found"),
            Self::InvalidListing => write!(f, "Invalid listing data"),
            Self::AmountRequired => write!(f, "Payment amount required"),
            Self::BelowMinimum => write!(f, "Payment amount below minimum"),
            Self::OrderNotFound => write!(f, "Order not found"),
            Self::InsufficientFunds => write!(f, "Insufficient funds"),
            Self::PaymentFailed => write!(f, "Payment failed"),
            Self::EscrowError(msg) => write!(f, "Escrow error: {}", msg),
            Self::SearchError(msg) => write!(f, "Search error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
            Self::InvalidToken => write!(f, "Invalid access token"),
            Self::TokenExpired => write!(f, "Token expired"),
            Self::DownloadLimitReached => write!(f, "Download limit reached"),
            Self::NoProviders => write!(f, "No content providers available"),
            Self::ContentNotFound => write!(f, "Content not found"),
            Self::InsufficientFundsForEscrow => write!(f, "Insufficient funds for escrow"),
            Self::EscrowExists => write!(f, "Escrow already exists"),
            Self::EscrowNotFound => write!(f, "Escrow not found"),
            Self::InvalidEscrowState => write!(f, "Invalid escrow state for operation"),
            Self::ReleaseConditionsNotMet => write!(f, "Release conditions not met"),
        }
    }
}

impl std::error::Error for MarketplaceError {}

/// Result type for marketplace operations.
pub type MarketplaceResult<T> = Result<T, MarketplaceError>;

/// Result type for commerce operations.
pub type CommerceResult<T> = Result<T, CommerceError>;
