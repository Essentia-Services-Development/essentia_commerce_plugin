//! Service types for order management.
//!
//! This module contains the OrderService and OrderFilter types that provide
//! the business logic and filtering capabilities for order management.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::basic_types::{OrderId, OrderCustomerId, OrderStatus, PaymentStatus, FulfillmentStatus};
use super::main_order_types::Order;

/// Order management service.
#[derive(Debug)]
pub struct OrderService {
    /// Orders indexed by ID.
    pub(crate) orders: Arc<Mutex<HashMap<OrderId, Order>>>,
    /// Orders indexed by customer.
    pub(crate) orders_by_customer: Arc<Mutex<HashMap<OrderCustomerId, Vec<OrderId>>>>,
    /// Order number counter.
    pub(crate) order_counter: Arc<Mutex<u64>>,
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
