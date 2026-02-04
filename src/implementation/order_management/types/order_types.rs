//! # Order Management Types - Order Types
//!
//! Type definitions for order-related structures including line items, payments, shipments, and history.

use std::collections::HashMap;

use crate::types::product_catalog::{Price, ProductId, Currency};
use crate::implementation::cart_system::{CartItem, ShippingAddress};

use super::basic_types::OrderStatus;

// ============================================================================
// ORDER LINE ITEM
// ============================================================================

/// Line item in an order.
#[derive(Debug, Clone)]
pub struct OrderLineItem {
    /// Line item ID.
    pub id: String,
    /// Product ID.
    pub product_id: ProductId,
    /// Variant ID.
    pub variant_id: Option<ProductId>,
    /// Product name.
    pub name: String,
    /// SKU.
    pub sku: String,
    /// Quantity ordered.
    pub quantity: u32,
    /// Quantity fulfilled.
    pub quantity_fulfilled: u32,
    /// Quantity refunded.
    pub quantity_refunded: u32,
    /// Unit price.
    pub unit_price: Price,
    /// Total before discount.
    pub subtotal: u64,
    /// Discount amount.
    pub discount: u64,
    /// Tax amount.
    pub tax: u64,
    /// Line total.
    pub total: u64,
    /// Product image URL.
    pub image_url: Option<String>,
    /// Whether item is taxable.
    pub taxable: bool,
    /// Whether item requires shipping.
    pub requires_shipping: bool,
    /// Custom properties.
    pub properties: HashMap<String, String>,
}

impl OrderLineItem {
    /// Creates a line item from a cart item.
    #[must_use]
    pub fn from_cart_item(item: &CartItem, line_id: String, tax_rate: f64) -> Self {
        let subtotal = item.subtotal();
        let discount = item.total_discount();
        let taxable_amount = subtotal.saturating_sub(discount);
        let tax = (taxable_amount as f64 * tax_rate / 100.0) as u64;
        let total = taxable_amount + tax;

        Self {
            id: line_id,
            product_id: item.product_id.clone(),
            variant_id: item.variant_id.clone(),
            name: item.product_name.to_string(),
            sku: item.product_sku.to_string(),
            quantity: item.quantity,
            quantity_fulfilled: 0,
            quantity_refunded: 0,
            unit_price: item.unit_price.clone(),
            subtotal,
            discount,
            tax,
            total,
            image_url: item.image_url.as_ref().map(|url| url.to_string()),
            taxable: true,
            requires_shipping: true,
            properties: item.custom_options.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        }
    }

    /// Quantity remaining to fulfill.
    #[must_use]
    pub fn quantity_remaining(&self) -> u32 {
        self.quantity.saturating_sub(self.quantity_fulfilled)
    }

    /// Whether item is fully fulfilled.
    #[must_use]
    pub fn is_fulfilled(&self) -> bool {
        self.quantity_fulfilled >= self.quantity
    }
}

// ============================================================================
// PAYMENT & TRANSACTION
// ============================================================================

/// Transaction status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Pending.
    Pending,
    /// Success.
    Success,
    /// Failed.
    Failed,
    /// Cancelled.
    Cancelled,
}

impl TransactionStatus {
    /// Display name.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Success => "Success",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }
}

/// Payment method used.
#[derive(Debug, Clone)]
pub struct PaymentMethod {
    /// Method identifier.
    pub id: String,
    /// Method type (card, crypto, etc).
    pub method_type: String,
    /// Last 4 digits (for cards).
    pub last_four: Option<String>,
    /// Card brand (Visa, Mastercard, etc).
    pub brand: Option<String>,
    /// Expiry month.
    pub exp_month: Option<u32>,
    /// Expiry year.
    pub exp_year: Option<u32>,
    /// Wallet address (for crypto).
    pub wallet_address: Option<String>,
}

/// Payment transaction record.
#[derive(Debug, Clone)]
pub struct PaymentTransaction {
    /// Transaction ID.
    pub id: String,
    /// External transaction reference.
    pub external_id: Option<String>,
    /// Transaction type.
    pub transaction_type: TransactionType,
    /// Amount.
    pub amount: u64,
    /// Currency.
    pub currency: Currency,
    /// Status.
    pub status: TransactionStatus,
    /// Gateway used.
    pub gateway: String,
    /// Payment method.
    pub payment_method: Option<PaymentMethod>,
    /// Error message if failed.
    pub error_message: Option<String>,
    /// Timestamp.
    pub created_at: u64,
}

/// Transaction type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    /// Authorization.
    Authorization,
    /// Capture.
    Capture,
    /// Refund.
    Refund,
    /// Void.
    Void,
}

// ============================================================================
// SHIPMENT & TRACKING
// ============================================================================

/// Shipment information.
#[derive(Debug, Clone)]
pub struct Shipment {
    /// Shipment ID.
    pub id: String,
    /// Carrier name.
    pub carrier: String,
    /// Tracking number.
    pub tracking_number: Option<String>,
    /// Tracking URL.
    pub tracking_url: Option<String>,
    /// Shipment status.
    pub status: ShipmentStatus,
    /// Items in this shipment.
    pub items: Vec<ShipmentItem>,
    /// Shipping address.
    pub shipping_address: ShippingAddress,
    /// Shipped date.
    pub shipped_at: Option<u64>,
    /// Delivered date.
    pub delivered_at: Option<u64>,
    /// Creation date.
    pub created_at: u64,
}

/// Shipment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShipmentStatus {
    /// Preparing shipment.
    #[default]
    Pending,
    /// Label created.
    LabelCreated,
    /// Picked up by carrier.
    PickedUp,
    /// In transit.
    InTransit,
    /// Out for delivery.
    OutForDelivery,
    /// Delivered.
    Delivered,
    /// Delivery failed.
    DeliveryFailed,
    /// Returned to sender.
    Returned,
}

/// Item in a shipment.
#[derive(Debug, Clone)]
pub struct ShipmentItem {
    /// Line item ID.
    pub line_item_id: String,
    /// Quantity shipped.
    pub quantity: u32,
}

// ============================================================================
// ORDER NOTES & HISTORY
// ============================================================================

/// Note attached to an order.
#[derive(Debug, Clone)]
pub struct OrderNote {
    /// Note ID.
    pub id: String,
    /// Note content.
    pub content: String,
    /// Whether visible to customer.
    pub customer_visible: bool,
    /// Author.
    pub author: String,
    /// Creation timestamp.
    pub created_at: u64,
}

impl OrderNote {
    /// Creates a new internal note.
    #[must_use]
    pub fn internal(content: impl Into<String>, author: impl Into<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            id: format!("note-{}", now),
            content: content.into(),
            customer_visible: false,
            author: author.into(),
            created_at: now,
        }
    }

    /// Creates a customer-visible note.
    #[must_use]
    pub fn customer_note(content: impl Into<String>, author: impl Into<String>) -> Self {
        let mut note = Self::internal(content, author);
        note.customer_visible = true;
        note
    }
}

/// Order history event.
#[derive(Debug, Clone)]
pub struct OrderHistoryEvent {
    /// Event ID.
    pub id: String,
    /// Event type.
    pub event_type: OrderEventType,
    /// Event description.
    pub description: String,
    /// Previous status (for status changes).
    pub previous_status: Option<OrderStatus>,
    /// New status (for status changes).
    pub new_status: Option<OrderStatus>,
    /// User who triggered the event.
    pub user: Option<String>,
    /// Timestamp.
    pub created_at: u64,
}

/// Order event type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderEventType {
    /// Order created.
    Created,
    /// Status changed.
    StatusChanged,
    /// Payment received.
    PaymentReceived,
    /// Payment failed.
    PaymentFailed,
    /// Shipped.
    Shipped,
    /// Delivered.
    Delivered,
    /// Cancelled.
    Cancelled,
    /// Refunded.
    Refunded,
    /// Note added.
    NoteAdded,
    /// Fulfillment updated.
    FulfillmentUpdated,
}
