//! Order implementation.
//!
//! Business logic implementations for the Order type.

use super::super::types::{
    basic_types::{FulfillmentStatus, OrderId, OrderStatus, PaymentStatus},
    main_order_types::{Order, OrderSource, OrderTotals},
    order_types::{
        OrderEventType, OrderHistoryEvent, OrderLineItem, OrderNote, PaymentTransaction, Shipment,
        TransactionStatus, TransactionType,
    },
};
use crate::implementation::cart_system::{Cart, ShippingMethod};

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
        let shipping_method =
            cart.shipping_method.clone().unwrap_or_else(ShippingMethod::free_shipping);

        let mut order = Self {
            id: order_id,
            order_number,
            customer_id: cart.customer_id.clone().into(),
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
        &mut self, event_type: OrderEventType, description: impl Into<String>, user: Option<String>,
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
                self.totals.amount_paid =
                    self.totals.amount_paid.saturating_add(transaction.amount);
            } else if transaction.transaction_type == TransactionType::Refund {
                self.totals.amount_refunded =
                    self.totals.amount_refunded.saturating_add(transaction.amount);
            }

            self.totals.amount_due = self
                .totals
                .grand_total
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
            (TransactionType::Capture, TransactionStatus::Success) => {
                OrderEventType::PaymentReceived
            },
            (TransactionType::Refund, TransactionStatus::Success) => OrderEventType::Refunded,
            (_, TransactionStatus::Failed) => OrderEventType::PaymentFailed,
            _ => OrderEventType::PaymentReceived,
        };

        self.add_history_event(
            event_type,
            format!(
                "Transaction {}: {}",
                transaction.id,
                transaction.status.display_name()
            ),
            None,
        );
        self.transactions.push(transaction);
        self.touch();
    }

    /// Adds a shipment.
    pub fn add_shipment(&mut self, shipment: Shipment) {
        // Update line item fulfillment quantities
        for ship_item in &shipment.items {
            if let Some(line_item) =
                self.line_items.iter_mut().find(|li| li.id == ship_item.line_item_id)
            {
                line_item.quantity_fulfilled =
                    line_item.quantity_fulfilled.saturating_add(ship_item.quantity);
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

        self.add_history_event(
            OrderEventType::Shipped,
            format!("Shipment {} created", shipment.id),
            None,
        );
        self.shipments.push(shipment);
        self.touch();
    }

    /// Adds a note to the order.
    pub fn add_note(&mut self, note: OrderNote) {
        self.add_history_event(
            OrderEventType::NoteAdded,
            "Note added",
            Some(note.author.clone()),
        );
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
