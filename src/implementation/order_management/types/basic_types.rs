//! # Order Management Types - Basic Types
//!
//! Core type definitions for order management including IDs, enums, and basic structs.

// ============================================================================
// BASIC IDENTIFIERS
// ============================================================================

/// Unique order identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderId(pub String);

impl OrderId {
    /// Creates a new order ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generates a new unique order ID.
    #[must_use]
    pub fn generate() -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        Self(format!("ORD-{}", timestamp))
    }
}

impl From<String> for OrderCustomerId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<crate::implementation::cart_system::CustomerId> for OrderCustomerId {
    fn from(customer_id: crate::implementation::cart_system::CustomerId) -> Self {
        Self(customer_id.0.to_string())
    }
}

/// Customer identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderCustomerId(pub String);

impl OrderCustomerId {
    /// Creates a new customer ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

// ============================================================================
// STATUS ENUMS
// ============================================================================

/// Order status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrderStatus {
    /// Order is pending payment.
    #[default]
    PendingPayment,
    /// Payment received, processing order.
    Processing,
    /// Order is on hold.
    OnHold,
    /// Order shipped.
    Shipped,
    /// Order delivered.
    Delivered,
    /// Order completed.
    Completed,
    /// Order cancelled.
    Cancelled,
    /// Order refunded.
    Refunded,
    /// Order failed.
    Failed,
    /// Partial refund issued.
    PartiallyRefunded,
}

impl OrderStatus {
    /// Whether order is cancellable.
    #[must_use]
    pub fn is_cancellable(&self) -> bool {
        matches!(self, Self::PendingPayment | Self::Processing | Self::OnHold)
    }

    /// Whether order is refundable.
    #[must_use]
    pub fn is_refundable(&self) -> bool {
        matches!(self, Self::Processing | Self::Shipped | Self::Delivered | Self::Completed)
    }

    /// Whether order is in a final state.
    #[must_use]
    pub fn is_final(&self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled | Self::Refunded | Self::Failed)
    }

    /// Display name.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::PendingPayment => "Pending Payment",
            Self::Processing => "Processing",
            Self::OnHold => "On Hold",
            Self::Shipped => "Shipped",
            Self::Delivered => "Delivered",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
            Self::Refunded => "Refunded",
            Self::Failed => "Failed",
            Self::PartiallyRefunded => "Partially Refunded",
        }
    }
}

/// Payment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaymentStatus {
    /// Awaiting payment.
    #[default]
    Pending,
    /// Payment authorized but not captured.
    Authorized,
    /// Payment captured.
    Captured,
    /// Payment partially refunded.
    PartiallyRefunded,
    /// Payment fully refunded.
    Refunded,
    /// Payment failed.
    Failed,
    /// Payment cancelled.
    Cancelled,
}

/// Fulfillment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FulfillmentStatus {
    /// Not yet fulfilled.
    #[default]
    Unfulfilled,
    /// Partially fulfilled.
    PartiallyFulfilled,
    /// Fully fulfilled.
    Fulfilled,
    /// Returned.
    Returned,
}
