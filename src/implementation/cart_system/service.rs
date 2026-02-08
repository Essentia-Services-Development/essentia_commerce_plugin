//! Cart management service

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::errors::CommerceError;

use super::cart::Cart;
use super::types::{CartId, CartStatus, CustomerId};

/// Cart management service.
#[derive(Debug)]
pub struct CartService {
    /// Carts indexed by ID.
    carts:             Arc<Mutex<HashMap<CartId, Cart>>>,
    /// Carts indexed by customer ID.
    carts_by_customer: Arc<Mutex<HashMap<CustomerId, Vec<CartId>>>>,
}

impl CartService {
    /// Creates a new cart service.
    #[must_use]
    pub fn new() -> Self {
        Self {
            carts:             Arc::new(Mutex::new(HashMap::new())),
            carts_by_customer: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Creates a new cart for a customer.
    pub fn create_cart(&self, customer_id: CustomerId) -> Result<Cart, CommerceError> {
        let cart = Cart::new(customer_id.clone());
        let cart_id = cart.id.clone();

        let mut carts = self.carts.lock().map_err(|_| CommerceError::LockError)?;
        let mut by_customer =
            self.carts_by_customer.lock().map_err(|_| CommerceError::LockError)?;

        carts.insert(cart_id.clone(), cart.clone());
        by_customer.entry(customer_id).or_insert_with(Vec::new).push(cart_id);

        Ok(cart)
    }

    /// Gets a cart by ID.
    pub fn get_cart(&self, id: &CartId) -> Result<Cart, CommerceError> {
        let carts = self.carts.lock().map_err(|_| CommerceError::LockError)?;
        carts
            .get(id)
            .cloned()
            .ok_or_else(|| CommerceError::CartNotFound(id.0.to_string()))
    }

    /// Gets active cart for a customer.
    pub fn get_customer_cart(
        &self, customer_id: &CustomerId,
    ) -> Result<Option<Cart>, CommerceError> {
        let carts = self.carts.lock().map_err(|_| CommerceError::LockError)?;
        let by_customer = self.carts_by_customer.lock().map_err(|_| CommerceError::LockError)?;

        let cart_ids = by_customer.get(customer_id).cloned().unwrap_or_default();

        // Return most recent active cart
        let active_cart = cart_ids
            .iter()
            .filter_map(|id| carts.get(id))
            .filter(|c| c.status == CartStatus::Active && !c.is_expired())
            .max_by_key(|c| c.last_activity_at)
            .cloned();

        Ok(active_cart)
    }

    /// Gets or creates a cart for a customer.
    pub fn get_or_create_cart(&self, customer_id: CustomerId) -> Result<Cart, CommerceError> {
        if let Some(cart) = self.get_customer_cart(&customer_id)? {
            return Ok(cart);
        }
        self.create_cart(customer_id)
    }

    /// Updates a cart.
    pub fn update_cart(&self, cart: Cart) -> Result<(), CommerceError> {
        let mut carts = self.carts.lock().map_err(|_| CommerceError::LockError)?;

        if !carts.contains_key(&cart.id) {
            return Err(CommerceError::CartNotFound(cart.id.0.to_string()));
        }

        carts.insert(cart.id.clone(), cart);
        Ok(())
    }

    /// Merges a guest cart into a customer cart.
    pub fn merge_carts(
        &self, guest_cart_id: &CartId, customer_id: &CustomerId,
    ) -> Result<Cart, CommerceError> {
        let carts = self.carts.lock().map_err(|_| CommerceError::LockError)?;

        let guest_cart = carts
            .get(guest_cart_id)
            .ok_or_else(|| CommerceError::CartNotFound(guest_cart_id.0.to_string()))?
            .clone();

        // Get or create customer cart
        drop(carts);
        let mut customer_cart = self.get_or_create_cart(customer_id.clone())?;

        // Merge items
        for item in guest_cart.items {
            if let Some(existing) =
                customer_cart.items.iter_mut().find(|i| i.product_id == item.product_id)
            {
                existing.quantity = existing.quantity.saturating_add(item.quantity);
            } else {
                customer_cart.items.push(item);
            }
        }

        // Update guest cart status
        let mut carts = self.carts.lock().map_err(|_| CommerceError::LockError)?;
        if let Some(guest) = carts.get_mut(guest_cart_id) {
            guest.status = CartStatus::Merged;
        }

        carts.insert(customer_cart.id.clone(), customer_cart.clone());
        Ok(customer_cart)
    }

    /// Marks cart as converted (after order creation).
    pub fn mark_as_converted(&self, cart_id: &CartId) -> Result<(), CommerceError> {
        let mut carts = self.carts.lock().map_err(|_| CommerceError::LockError)?;

        let cart = carts
            .get_mut(cart_id)
            .ok_or_else(|| CommerceError::CartNotFound(cart_id.0.to_string()))?;

        cart.status = CartStatus::Converted;
        Ok(())
    }

    /// Deletes expired and abandoned carts.
    pub fn cleanup_carts(&self, max_age_days: u64) -> Result<usize, CommerceError> {
        let mut carts = self.carts.lock().map_err(|_| CommerceError::LockError)?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let max_age_secs = max_age_days * 24 * 60 * 60;
        let initial_count = carts.len();

        carts.retain(|_, cart| {
            let age = now.saturating_sub(cart.last_activity_at);
            let is_old = age > max_age_secs;
            let is_inactive = matches!(
                cart.status,
                CartStatus::Converted | CartStatus::Merged | CartStatus::Expired
            );
            !is_old || !is_inactive
        });

        Ok(initial_count - carts.len())
    }
}

impl Default for CartService {
    fn default() -> Self {
        Self::new()
    }
}
