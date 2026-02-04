//! Service implementation.
//!
//! Business logic implementations for the OrderService type.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::super::types::basic_types::{OrderId, OrderCustomerId, OrderStatus};
use super::super::types::main_order_types::Order;
use super::super::types::order_types::OrderNote;
use super::super::types::service_types::{OrderService, OrderFilter};
use crate::implementation::cart_system::Cart;
use crate::errors::CommerceError;

    impl OrderService {
        /// Creates a new order service.
        #[must_use]
        pub fn new() -> Self {
            Self {
                orders: Arc::new(Mutex::new(HashMap::new())),
                orders_by_customer: Arc::new(Mutex::new(HashMap::new())),
                order_counter: Arc::new(Mutex::new(1000)),
            }
        }

        /// Generates the next order number.
        fn next_order_number(&self) -> u64 {
        let mut counter = self.order_counter.lock().unwrap_or_else(|e: std::sync::PoisonError<std::sync::MutexGuard<'_, u64>>| e.into_inner());
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
