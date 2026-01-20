//! # Order Management (GAP-220-D-003)
//!
//! Complete order lifecycle management for the e-commerce platform.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::errors::CommerceError;
use crate::r#impl::product_catalog::{Price, ProductId, Currency};
use crate::r#impl::cart_system::{Cart, CartItem, ShippingAddress, ShippingMethod, CartTotals};

// Payment plugin integration
use essentia_payment_plugin::{
    PaymentPlugin, PaymentAmount, PaymentStatus as PluginPaymentStatus,
};

// Blockchain plugin integration (for transaction settlement)
use essentia_blockchain_plugin::{
    BlockchainPlugin, Transaction as BlockchainTransaction, TransactionStatus as BlockchainTxStatus,
};

// ============================================================================
// CORE TYPES
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

impl std::fmt::Display for OrderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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

/// Transaction status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Pending.
    Pending,
    /// Successful.
    Success,
    /// Failed.
    Failed,
    /// Cancelled.
    Cancelled,
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
    /// Refund issued.
    RefundIssued,
    /// Shipment created.
    ShipmentCreated,
    /// Shipment updated.
    ShipmentUpdated,
    /// Note added.
    NoteAdded,
    /// Item cancelled.
    ItemCancelled,
    /// Email sent.
    EmailSent,
}

// ============================================================================
// ORDER
// ============================================================================

/// Complete order.
#[derive(Debug, Clone)]
pub struct Order {
    /// Order ID.
    pub id: OrderId,
    /// Order number (display).
    pub order_number: String,
    /// Customer ID.
    pub customer_id: OrderCustomerId,
    /// Customer email.
    pub customer_email: String,
    /// Customer phone.
    pub customer_phone: Option<String>,
    /// Order status.
    pub status: OrderStatus,
    /// Payment status.
    pub payment_status: PaymentStatus,
    /// Fulfillment status.
    pub fulfillment_status: FulfillmentStatus,
    /// Line items.
    pub line_items: Vec<OrderLineItem>,
    /// Shipping address.
    pub shipping_address: ShippingAddress,
    /// Billing address.
    pub billing_address: Option<ShippingAddress>,
    /// Shipping method.
    pub shipping_method: ShippingMethod,
    /// Order totals.
    pub totals: OrderTotals,
    /// Currency.
    pub currency: Currency,
    /// Payment transactions.
    pub transactions: Vec<PaymentTransaction>,
    /// Payment invoice ID (from payment plugin).
    pub payment_invoice_id: Option<String>,
    /// Blockchain transaction ID (for settlement).
    pub blockchain_tx_id: Option<[u8; 32]>,
    /// Shipments.
    pub shipments: Vec<Shipment>,
    /// Order notes.
    pub notes: Vec<OrderNote>,
    /// Order history.
    pub history: Vec<OrderHistoryEvent>,
    /// Customer note at checkout.
    pub customer_note: Option<String>,
    /// IP address.
    pub ip_address: Option<String>,
    /// User agent.
    pub user_agent: Option<String>,
    /// Source channel.
    pub source: OrderSource,
    /// Tags.
    pub tags: Vec<String>,
    /// Creation timestamp.
    pub created_at: u64,
    /// Last update timestamp.
    pub updated_at: u64,
}

/// Order totals.
#[derive(Debug, Clone, Default)]
pub struct OrderTotals {
    /// Subtotal.
    pub subtotal: u64,
    /// Total discounts.
    pub discount_total: u64,
    /// Shipping total.
    pub shipping_total: u64,
    /// Tax total.
    pub tax_total: u64,
    /// Grand total.
    pub grand_total: u64,
    /// Amount paid.
    pub amount_paid: u64,
    /// Amount refunded.
    pub amount_refunded: u64,
    /// Amount due.
    pub amount_due: u64,
}

impl OrderTotals {
    /// Creates totals from cart totals.
    #[must_use]
    pub fn from_cart_totals(totals: &CartTotals) -> Self {
        Self {
            subtotal: totals.subtotal,
            discount_total: totals.discount_total,
            shipping_total: totals.shipping_total,
            tax_total: totals.tax_total,
            grand_total: totals.grand_total,
            amount_paid: 0,
            amount_refunded: 0,
            amount_due: totals.grand_total,
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

impl Order {
    /// Creates an order from a cart.
    #[must_use]
    pub fn from_cart(cart: &Cart, customer_email: impl Into<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let order_id = OrderId::generate();
        let order_number = format!("#{}", &order_id.0[4..]);

        let cart_totals = cart.calculate_totals();

        // Convert cart items to order line items
        let line_items: Vec<OrderLineItem> = cart
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                OrderLineItem::from_cart_item(item, format!("line-{}", i + 1), cart.tax_rate)
            })
            .collect();

        let totals = OrderTotals::from_cart_totals(&cart_totals);

        let shipping_address = cart.shipping_address.clone().unwrap_or_default();
        let shipping_method = cart.shipping_method.clone().unwrap_or_else(ShippingMethod::free_shipping);

        let mut order = Self {
            id: order_id,
            order_number,
            customer_id: OrderCustomerId::new(cart.customer_id.0.to_string()),
            customer_email: customer_email.into(),
            customer_phone: None,
            status: OrderStatus::PendingPayment,
            payment_status: PaymentStatus::Pending,
            fulfillment_status: FulfillmentStatus::Unfulfilled,
            line_items,
            shipping_address,
            billing_address: cart.billing_address.clone(),
            shipping_method,
            totals,
            currency: cart.currency.clone(),
            transactions: Vec::new(),
            payment_invoice_id: None,
            blockchain_tx_id: None,
            shipments: Vec::new(),
            notes: Vec::new(),
            history: Vec::new(),
            customer_note: cart.notes.as_ref().map(|n| n.to_string()),
            ip_address: None,
            user_agent: None,
            source: OrderSource::Web,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        };

        // Add creation event
        order.add_history_event(OrderEventType::Created, "Order created", None);

        order
    }

    /// Adds a history event.
    pub fn add_history_event(
        &mut self,
        event_type: OrderEventType,
        description: impl Into<String>,
        user: Option<String>,
    ) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.history.push(OrderHistoryEvent {
            id: format!("event-{}", now),
            event_type,
            description: description.into(),
            previous_status: None,
            new_status: None,
            user,
            created_at: now,
        });
    }

    /// Updates order status.
    pub fn update_status(&mut self, new_status: OrderStatus, user: Option<String>) {
        let previous_status = self.status;
        self.status = new_status;
        self.touch();

        self.history.push(OrderHistoryEvent {
            id: format!("event-{}", self.updated_at),
            event_type: OrderEventType::StatusChanged,
            description: format!(
                "Status changed from {} to {}",
                previous_status.display_name(),
                new_status.display_name()
            ),
            previous_status: Some(previous_status),
            new_status: Some(new_status),
            user,
            created_at: self.updated_at,
        });
    }

    /// Records a payment.
    pub fn record_payment(&mut self, transaction: PaymentTransaction) {
        if transaction.status == TransactionStatus::Success {
            if transaction.transaction_type == TransactionType::Capture {
                self.totals.amount_paid = self.totals.amount_paid.saturating_add(transaction.amount);
            } else if transaction.transaction_type == TransactionType::Refund {
                self.totals.amount_refunded = self.totals.amount_refunded.saturating_add(transaction.amount);
            }

            self.totals.amount_due = self.totals.grand_total
                .saturating_sub(self.totals.amount_paid)
                .saturating_add(self.totals.amount_refunded);

            // Update payment status
            if self.totals.amount_refunded >= self.totals.grand_total {
                self.payment_status = PaymentStatus::Refunded;
            } else if self.totals.amount_refunded > 0 {
                self.payment_status = PaymentStatus::PartiallyRefunded;
            } else if self.totals.amount_paid >= self.totals.grand_total {
                self.payment_status = PaymentStatus::Captured;
            }
        }

        let event_type = match (transaction.transaction_type, transaction.status) {
            (TransactionType::Capture, TransactionStatus::Success) => OrderEventType::PaymentReceived,
            (TransactionType::Refund, TransactionStatus::Success) => OrderEventType::RefundIssued,
            (_, TransactionStatus::Failed) => OrderEventType::PaymentFailed,
            _ => OrderEventType::PaymentReceived,
        };

        self.add_history_event(event_type, format!("Transaction {}: {}", transaction.id, transaction.status.display_name()), None);
        self.transactions.push(transaction);
        self.touch();
    }

    /// Adds a shipment.
    pub fn add_shipment(&mut self, shipment: Shipment) {
        // Update line item fulfillment quantities
        for ship_item in &shipment.items {
            if let Some(line_item) = self.line_items.iter_mut().find(|li| li.id == ship_item.line_item_id) {
                line_item.quantity_fulfilled = line_item.quantity_fulfilled.saturating_add(ship_item.quantity);
            }
        }

        // Update fulfillment status
        let total_items: u32 = self.line_items.iter().map(|i| i.quantity).sum();
        let fulfilled_items: u32 = self.line_items.iter().map(|i| i.quantity_fulfilled).sum();

        self.fulfillment_status = if fulfilled_items == 0 {
            FulfillmentStatus::Unfulfilled
        } else if fulfilled_items >= total_items {
            FulfillmentStatus::Fulfilled
        } else {
            FulfillmentStatus::PartiallyFulfilled
        };

        self.add_history_event(OrderEventType::ShipmentCreated, format!("Shipment {} created", shipment.id), None);
        self.shipments.push(shipment);
        self.touch();
    }

    /// Adds a note to the order.
    pub fn add_note(&mut self, note: OrderNote) {
        self.add_history_event(OrderEventType::NoteAdded, "Note added", Some(note.author.clone()));
        self.notes.push(note);
        self.touch();
    }

    /// Whether order can be cancelled.
    #[must_use]
    pub fn can_cancel(&self) -> bool {
        self.status.is_cancellable()
    }

    /// Whether order can be refunded.
    #[must_use]
    pub fn can_refund(&self) -> bool {
        self.status.is_refundable() && self.totals.amount_paid > self.totals.amount_refunded
    }

    /// Maximum refundable amount.
    #[must_use]
    pub fn max_refund_amount(&self) -> u64 {
        self.totals.amount_paid.saturating_sub(self.totals.amount_refunded)
    }

    /// Updates the timestamp.
    fn touch(&mut self) {
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }
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

// ============================================================================
// ORDER SERVICE
// ============================================================================

/// Order management service.
#[derive(Debug)]
pub struct OrderService {
    /// Orders indexed by ID.
    orders: Arc<Mutex<HashMap<OrderId, Order>>>,
    /// Orders indexed by customer.
    orders_by_customer: Arc<Mutex<HashMap<OrderCustomerId, Vec<OrderId>>>>,
    /// Order number counter.
    order_counter: Arc<Mutex<u64>>,
    /// Payment plugin for processing payments.
    payment_plugin: Option<PaymentPlugin>,
    /// Blockchain plugin for transaction settlement.
    blockchain_plugin: Option<BlockchainPlugin>,
}

impl OrderService {
    /// Creates a new order service.
    #[must_use]
    pub fn new() -> Self {
        Self {
            orders: Arc::new(Mutex::new(HashMap::new())),
            orders_by_customer: Arc::new(Mutex::new(HashMap::new())),
            order_counter: Arc::new(Mutex::new(1000)),
            payment_plugin: None,
            blockchain_plugin: None,
        }
    }

    /// Creates a new order service with payment and blockchain plugins.
    #[must_use]
    pub fn with_plugins(payment_plugin: PaymentPlugin, blockchain_plugin: BlockchainPlugin) -> Self {
        Self {
            orders: Arc::new(Mutex::new(HashMap::new())),
            orders_by_customer: Arc::new(Mutex::new(HashMap::new())),
            order_counter: Arc::new(Mutex::new(1000)),
            payment_plugin: Some(payment_plugin),
            blockchain_plugin: Some(blockchain_plugin),
        }
    }

    /// Generates the next order number.
    fn next_order_number(&self) -> u64 {
        let mut counter = self.order_counter.lock().unwrap_or_else(|e| e.into_inner());
        let num = *counter;
        *counter += 1;
        num
    }

    /// Creates an order from a cart.
    pub fn create_order(&self, cart: &Cart, customer_email: impl Into<String>) -> Result<Order, CommerceError> {
        cart.validate_for_checkout()?;

        let mut order = Order::from_cart(cart, customer_email);

        // Use sequential order number
        order.order_number = format!("#{}", self.next_order_number());

        let order_id = order.id.clone();
        let customer_id = order.customer_id.clone();

        let mut orders = self.orders.lock().map_err(|_| CommerceError::LockError)?;
        let mut by_customer = self.orders_by_customer.lock().map_err(|_| CommerceError::LockError)?;

        orders.insert(order_id.clone(), order.clone());
        by_customer
            .entry(customer_id)
            .or_insert_with(Vec::new)
            .push(order_id);

        Ok(order)
    }

    /// Gets an order by ID.
    pub fn get_order(&self, id: &OrderId) -> Result<Order, CommerceError> {
        let orders = self.orders.lock().map_err(|_| CommerceError::LockError)?;
        orders
            .get(id)
            .cloned()
            .ok_or_else(|| CommerceError::OrderNotFound(id.0.clone()))
    }

    /// Gets orders for a customer.
    pub fn get_customer_orders(&self, customer_id: &OrderCustomerId) -> Result<Vec<Order>, CommerceError> {
        let orders = self.orders.lock().map_err(|_| CommerceError::LockError)?;
        let by_customer = self.orders_by_customer.lock().map_err(|_| CommerceError::LockError)?;

        let order_ids = by_customer.get(customer_id).cloned().unwrap_or_default();
        let mut customer_orders: Vec<Order> = order_ids
            .iter()
            .filter_map(|id| orders.get(id).cloned())
            .collect();

        // Sort by creation date descending
        customer_orders.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(customer_orders)
    }

    /// Updates an order.
    pub fn update_order(&self, order: Order) -> Result<(), CommerceError> {
        let mut orders = self.orders.lock().map_err(|_| CommerceError::LockError)?;

        if !orders.contains_key(&order.id) {
            return Err(CommerceError::OrderNotFound(order.id.0.clone()));
        }

        orders.insert(order.id.clone(), order);
        Ok(())
    }

    /// Creates a payment invoice for an order.
    pub fn create_payment_invoice(&self, order_id: &OrderId) -> Result<String, CommerceError> {
        let mut orders = self.orders.lock().map_err(|_| CommerceError::LockError)?;
        let order = orders.get_mut(order_id).ok_or_else(|| CommerceError::OrderNotFound(order_id.0.clone()))?;

        let payment_plugin = self.payment_plugin.as_ref().ok_or(CommerceError::PaymentPluginNotConfigured)?;

        // Create payment invoice
        let amount = PaymentAmount::from_satoshis(order.totals.grand_total);
        let description = format!("Order {} - {}", order.order_number, order.customer_email);

        // Generate invoice through payment plugin
        let invoice = payment_plugin.create_invoice(Some(amount.satoshis), description)
            .map_err(|e| CommerceError::PaymentError(format!("Failed to generate invoice: {:?}", e)))?;

        // Store invoice ID in order
        order.payment_invoice_id = Some(invoice.encoded.clone());

        Ok(invoice.encoded)
    }

    /// Processes a payment for an order.
    pub fn process_payment(&self, order_id: &OrderId, payment_hash: [u8; 32]) -> Result<(), CommerceError> {
        let mut orders = self.orders.lock().map_err(|_| CommerceError::LockError)?;
        let order = orders.get_mut(order_id).ok_or_else(|| CommerceError::OrderNotFound(order_id.0.clone()))?;

        let payment_plugin = self.payment_plugin.as_ref().ok_or(CommerceError::PaymentPluginNotConfigured)?;

        // Check payment status
        let status = payment_plugin.get_payment_status(&payment_hash)
            .map_err(|e| CommerceError::PaymentError(format!("Failed to get payment status: {:?}", e)))?;

        match status {
            PluginPaymentStatus::Succeeded => {
                // Payment successful - record transaction and update order
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);

                let transaction = PaymentTransaction {
                    id: format!("txn-{}", payment_hash.iter().fold(String::new(), |mut acc, b| { acc.push_str(&format!("{:02x}", b)); acc })),
                    external_id: Some(payment_hash.iter().fold(String::new(), |mut acc, b| { acc.push_str(&format!("{:02x}", b)); acc })),
                    transaction_type: TransactionType::Capture,
                    amount: order.totals.grand_total,
                    currency: order.currency.clone(),
                    status: TransactionStatus::Success,
                    gateway: "lightning".to_string(),
                    payment_method: None,
                    error_message: None,
                    created_at: now,
                };

                order.record_payment(transaction);
                order.update_status(OrderStatus::Processing, Some("payment_system".to_string()));

                // Create blockchain transaction for settlement if plugin available
                if let Some(blockchain_plugin) = &self.blockchain_plugin {
                    let blockchain_tx = BlockchainTransaction {
                        id: payment_hash,
                        sender: [0u8; 32], // Will be set by merchant
                        recipient: [0u8; 32], // Will be set by merchant
                        amount: order.totals.grand_total,
                        fee: 1000, // Default fee
                        signature: Vec::new(),
                        status: BlockchainTxStatus::Pending,
                        timestamp: now,
                    };

                    let tx = blockchain_plugin.submit_transaction(blockchain_tx)
                        .map_err(|e| CommerceError::BlockchainError(format!("Failed to submit blockchain transaction: {:?}", e)))?;

                    order.blockchain_tx_id = Some(tx.id);
                }

                Ok(())
            }
            PluginPaymentStatus::Failed => {
                // Payment failed
                order.update_status(OrderStatus::Failed, Some("payment_system".to_string()));
                Err(CommerceError::PaymentFailed("Payment failed".to_string()))
            }
            _ => {
                // Payment still pending
                Ok(())
            }
        }
    }

    /// Updates the status of an order.
    pub fn update_order_status(&self, order_id: &OrderId, status: OrderStatus, user: Option<String>) -> Result<(), CommerceError> {
        let mut orders = self.orders.lock().map_err(|_| CommerceError::LockError)?;

        let order = orders
            .get_mut(order_id)
            .ok_or_else(|| CommerceError::OrderNotFound(order_id.0.clone()))?;

        order.update_status(status, user);
        Ok(())
    }

    /// Cancels an order.
    pub fn cancel_order(&self, order_id: &OrderId, reason: impl Into<String>) -> Result<(), CommerceError> {
        let mut orders = self.orders.lock().map_err(|_| CommerceError::LockError)?;

        let order = orders
            .get_mut(order_id)
            .ok_or_else(|| CommerceError::OrderNotFound(order_id.0.clone()))?;

        if !order.can_cancel() {
            return Err(CommerceError::OrderNotCancellable(order_id.0.clone()));
        }

        order.update_status(OrderStatus::Cancelled, None);
        order.add_note(OrderNote::internal(format!("Order cancelled: {}", reason.into()), "System"));

        Ok(())
    }

    /// Searches orders.
    pub fn search_orders(&self, filter: &OrderFilter) -> Result<Vec<Order>, CommerceError> {
        let orders = self.orders.lock().map_err(|_| CommerceError::LockError)?;

        let filtered: Vec<Order> = orders
            .values()
            .filter(|o| self.matches_filter(o, filter))
            .cloned()
            .collect();

        Ok(filtered)
    }

    /// Matches order against filter.
    fn matches_filter(&self, order: &Order, filter: &OrderFilter) -> bool {
        if let Some(status) = filter.status && order.status != status {
            return false;
        }

        if let Some(payment_status) = filter.payment_status && order.payment_status != payment_status {
            return false;
        }

        if let Some(fulfillment_status) = filter.fulfillment_status && order.fulfillment_status != fulfillment_status {
            return false;
        }

        if let Some(min_total) = filter.min_total && order.totals.grand_total < min_total {
            return false;
        }

        if let Some(max_total) = filter.max_total && order.totals.grand_total > max_total {
            return false;
        }

        if let Some(from) = filter.created_from && order.created_at < from {
            return false;
        }

        if let Some(to) = filter.created_to && order.created_at > to {
            return false;
        }

        true
    }
}

impl Default for OrderService {
    fn default() -> Self {
        Self::new()
    }
}

/// Order search filter.
#[derive(Debug, Clone, Default)]
pub struct OrderFilter {
    /// Filter by status.
    pub status: Option<OrderStatus>,
    /// Filter by payment status.
    pub payment_status: Option<PaymentStatus>,
    /// Filter by fulfillment status.
    pub fulfillment_status: Option<FulfillmentStatus>,
    /// Minimum total.
    pub min_total: Option<u64>,
    /// Maximum total.
    pub max_total: Option<u64>,
    /// Created from timestamp.
    pub created_from: Option<u64>,
    /// Created to timestamp.
    pub created_to: Option<u64>,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r#impl::cart_system::{Cart, CustomerId};
    use crate::r#impl::product_catalog::{Product, ProductId, Sku, ProductStatus, Price, Currency};

    fn create_test_cart() -> Cart {
        let mut cart = Cart::new(CustomerId::new("customer-1"));
        cart.tax_rate = 10.0;

        let mut product = Product::new(
            ProductId::new("prod-001"),
            Sku::new("SKU-001"),
            "Test Product",
        );
        product.status = ProductStatus::Active;
        product.price = Price::new(1000, Currency::usd(), 2);
        product.inventory_quantity = 100;

        cart.add_item(&product, 2).expect("add item");
        cart.set_shipping_address(ShippingAddress::new(
            "John", "Doe", "123 Main St", "City", "State", "12345", "US",
        ));
        cart.set_shipping_method(ShippingMethod::free_shipping());

        cart
    }

    #[test]
    fn test_order_creation() {
        let cart = create_test_cart();
        let order = Order::from_cart(&cart, "test@example.com");

        assert_eq!(order.status, OrderStatus::PendingPayment);
        assert_eq!(order.payment_status, PaymentStatus::Pending);
        assert_eq!(order.fulfillment_status, FulfillmentStatus::Unfulfilled);
        assert_eq!(order.line_items.len(), 1);
        assert_eq!(order.line_items[0].quantity, 2);
    }

    #[test]
    fn test_order_status_update() {
        let cart = create_test_cart();
        let mut order = Order::from_cart(&cart, "test@example.com");

        order.update_status(OrderStatus::Processing, Some("admin".to_string()));

        assert_eq!(order.status, OrderStatus::Processing);
        assert!(order.history.iter().any(|e| e.event_type == OrderEventType::StatusChanged));
    }

    #[test]
    fn test_payment_recording() {
        let cart = create_test_cart();
        let mut order = Order::from_cart(&cart, "test@example.com");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let transaction = PaymentTransaction {
            id: "txn-001".to_string(),
            external_id: Some("stripe-123".to_string()),
            transaction_type: TransactionType::Capture,
            amount: order.totals.grand_total,
            currency: Currency::usd(),
            status: TransactionStatus::Success,
            gateway: "stripe".to_string(),
            payment_method: None,
            error_message: None,
            created_at: now,
        };

        order.record_payment(transaction);

        assert_eq!(order.payment_status, PaymentStatus::Captured);
        assert_eq!(order.totals.amount_paid, order.totals.grand_total);
        assert_eq!(order.totals.amount_due, 0);
    }

    #[test]
    fn test_order_cancellation() {
        let cart = create_test_cart();
        let mut order = Order::from_cart(&cart, "test@example.com");

        assert!(order.can_cancel());
        order.update_status(OrderStatus::Cancelled, None);
        assert!(!order.can_cancel());
    }

    #[test]
    fn test_order_service() {
        let service = OrderService::new();
        let cart = create_test_cart();

        let order = service.create_order(&cart, "test@example.com").expect("create");
        let retrieved = service.get_order(&order.id).expect("get");

        assert_eq!(order.id, retrieved.id);
    }

    #[test]
    fn test_customer_orders() {
        let service = OrderService::new();
        let cart = create_test_cart();

        let order1 = service.create_order(&cart, "test@example.com").expect("create 1");
        let _order2 = service.create_order(&cart, "test@example.com").expect("create 2");

        let customer_id = order1.customer_id.clone();
        let orders = service.get_customer_orders(&customer_id).expect("get orders");

        assert_eq!(orders.len(), 2);
    }
}
