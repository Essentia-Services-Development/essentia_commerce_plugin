//! Main order types for the order management system.
//!
//! This module contains the core Order struct and related types that define
//! the complete order data model.

use super::{
    basic_types::{FulfillmentStatus, OrderCustomerId, OrderId, OrderStatus, PaymentStatus},
    order_types::{OrderHistoryEvent, OrderLineItem, OrderNote, PaymentTransaction, Shipment},
};
use crate::{
    implementation::cart_system::{ShippingAddress, ShippingMethod},
    types::product_catalog::Currency,
};

/// Complete order.
#[derive(Debug, Clone)]
pub struct Order {
    /// Order ID.
    pub id:                 OrderId,
    /// Order number (display).
    pub order_number:       String,
    /// Customer ID.
    pub customer_id:        OrderCustomerId,
    /// Customer email.
    pub customer_email:     String,
    /// Customer phone.
    pub customer_phone:     Option<String>,
    /// Order status.
    pub status:             OrderStatus,
    /// Payment status.
    pub payment_status:     PaymentStatus,
    /// Fulfillment status.
    pub fulfillment_status: FulfillmentStatus,
    /// Line items.
    pub line_items:         Vec<OrderLineItem>,
    /// Shipping address.
    pub shipping_address:   ShippingAddress,
    /// Billing address.
    pub billing_address:    Option<ShippingAddress>,
    /// Shipping method.
    pub shipping_method:    ShippingMethod,
    /// Order totals.
    pub totals:             OrderTotals,
    /// Currency.
    pub currency:           Currency,
    /// Payment transactions.
    pub transactions:       Vec<PaymentTransaction>,
    /// Payment invoice ID (from payment plugin).
    pub payment_invoice_id: Option<String>,
    /// Blockchain transaction ID (for settlement).
    pub blockchain_tx_id:   Option<[u8; 32]>,
    /// Shipments.
    pub shipments:          Vec<Shipment>,
    /// Order notes.
    pub notes:              Vec<OrderNote>,
    /// Order history.
    pub history:            Vec<OrderHistoryEvent>,
    /// Customer note at checkout.
    pub customer_note:      Option<String>,
    /// IP address.
    pub ip_address:         Option<String>,
    /// User agent.
    pub user_agent:         Option<String>,
    /// Source channel.
    pub source:             OrderSource,
    /// Tags.
    pub tags:               Vec<String>,
    /// Creation timestamp.
    pub created_at:         u64,
    /// Last update timestamp.
    pub updated_at:         u64,
}

/// Order totals.
#[derive(Debug, Clone, Default)]
pub struct OrderTotals {
    /// Subtotal.
    pub subtotal:        u64,
    /// Total discounts.
    pub discount_total:  u64,
    /// Shipping total.
    pub shipping_total:  u64,
    /// Tax total.
    pub tax_total:       u64,
    /// Grand total.
    pub grand_total:     u64,
    /// Amount paid.
    pub amount_paid:     u64,
    /// Amount refunded.
    pub amount_refunded: u64,
    /// Amount due.
    pub amount_due:      u64,
}

impl OrderTotals {
    /// Creates totals from cart totals.
    #[must_use]
    pub fn from_cart_totals(totals: &crate::implementation::cart_system::CartTotals) -> Self {
        Self {
            subtotal:        totals.subtotal,
            discount_total:  totals.discount_total,
            shipping_total:  totals.shipping_total,
            tax_total:       totals.tax_total,
            grand_total:     totals.grand_total,
            amount_paid:     0,
            amount_refunded: 0,
            amount_due:      totals.grand_total,
        }
    }
}

/// Order source channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrderSource {
    /// Web store.
    #[default]
    Web,
    /// Mobile app.
    Mobile,
    /// API.
    Api,
    /// Point of sale.
    Pos,
    /// Manual/admin.
    Manual,
    /// Import.
    Import,
}
