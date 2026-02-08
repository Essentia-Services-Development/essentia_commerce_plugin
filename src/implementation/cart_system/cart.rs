//! Shopping cart and totals

use std::borrow::Cow;

use crate::{
    errors::CommerceError,
    types::product_catalog::{Currency, Product, ProductId},
};

use super::item::CartItem;
use super::shipping::{ShippingAddress, ShippingMethod};
use super::types::{AppliedDiscount, CartId, CartStatus, CustomerId, DiscountType};

/// Cart price totals.
#[derive(Debug, Clone, Default)]
pub struct CartTotals {
    /// Subtotal (sum of line totals before discounts).
    pub subtotal:       u64,
    /// Total discounts applied.
    pub discount_total: u64,
    /// Shipping cost.
    pub shipping_total: u64,
    /// Tax amount.
    pub tax_total:      u64,
    /// Grand total.
    pub grand_total:    u64,
    /// Total savings (from sales and discounts).
    pub total_savings:  u64,
    /// Number of items.
    pub item_count:     u32,
    /// Currency.
    pub currency:       Currency,
}

impl CartTotals {
    /// Calculates totals for a cart.
    #[must_use]
    pub fn calculate(
        items: &[CartItem], cart_discounts: &[AppliedDiscount], shipping: Option<&ShippingMethod>,
        tax_rate: f64, currency: Currency,
    ) -> Self {
        let subtotal: u64 = items.iter().map(|i| i.subtotal()).sum();
        let item_discounts: u64 = items.iter().map(|i| i.total_discount()).sum();
        let sale_savings: u64 = items.iter().map(|i| i.sale_savings()).sum();

        // Calculate cart-level discounts
        let mut cart_discount_total: u64 = 0;
        for discount in cart_discounts {
            match discount.discount_type {
                DiscountType::Percentage => {
                    cart_discount_total += (subtotal * discount.value) / 100;
                },
                DiscountType::FixedAmount => {
                    cart_discount_total += discount.value;
                },
                DiscountType::FreeShipping | DiscountType::BuyXGetY => {
                    // Handled separately
                },
            }
        }

        let discount_total = item_discounts + cart_discount_total;
        let subtotal_after_discount = subtotal.saturating_sub(discount_total);

        // Check for free shipping discount
        let has_free_shipping =
            cart_discounts.iter().any(|d| d.discount_type == DiscountType::FreeShipping);

        let shipping_total = if has_free_shipping {
            0
        } else {
            shipping.map(|s| s.cost.amount).unwrap_or(0)
        };

        // Calculate tax
        let tax_total = ((subtotal_after_discount as f64) * tax_rate / 100.0) as u64;

        let grand_total = subtotal_after_discount + shipping_total + tax_total;
        let total_savings = sale_savings + discount_total;

        let item_count: u32 = items.iter().map(|i| i.quantity).sum();

        Self {
            subtotal,
            discount_total,
            shipping_total,
            tax_total,
            grand_total,
            total_savings,
            item_count,
            currency,
        }
    }
}

/// Shopping cart.
#[derive(Debug, Clone)]
pub struct Cart {
    /// Cart ID.
    pub id:               CartId,
    /// Customer ID.
    pub customer_id:      CustomerId,
    /// Cart status.
    pub status:           CartStatus,
    /// Items in cart.
    pub items:            Vec<CartItem>,
    /// Applied coupon codes.
    pub discounts:        Vec<AppliedDiscount>,
    /// Shipping address.
    pub shipping_address: Option<ShippingAddress>,
    /// Billing address.
    pub billing_address:  Option<ShippingAddress>,
    /// Selected shipping method.
    pub shipping_method:  Option<ShippingMethod>,
    /// Default currency.
    pub currency:         Currency,
    /// Tax rate percentage.
    pub tax_rate:         f64,
    /// Cart notes.
    pub notes:            Option<Cow<'static, str>>,
    /// Creation timestamp.
    pub created_at:       u64,
    /// Last update timestamp.
    pub updated_at:       u64,
    /// Last activity timestamp.
    pub last_activity_at: u64,
    /// Cart expiration timestamp.
    pub expires_at:       Option<u64>,
}

impl Cart {
    /// Creates a new cart.
    #[must_use]
    pub fn new(customer_id: CustomerId) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            id: CartId::generate(),
            customer_id,
            status: CartStatus::Active,
            items: Vec::new(),
            discounts: Vec::new(),
            shipping_address: None,
            billing_address: None,
            shipping_method: None,
            currency: Currency::usd(),
            tax_rate: 0.0,
            notes: None,
            created_at: now,
            updated_at: now,
            last_activity_at: now,
            expires_at: Some(now + 7 * 24 * 60 * 60), // 7 days default
        }
    }

    /// Creates a guest cart.
    #[must_use]
    pub fn guest() -> Self {
        Self::new(CustomerId::guest())
    }

    /// Whether cart is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Number of unique items.
    #[must_use]
    pub fn unique_item_count(&self) -> usize {
        self.items.len()
    }

    /// Total quantity of all items.
    #[must_use]
    pub fn total_quantity(&self) -> u32 {
        self.items.iter().map(|i| i.quantity).sum()
    }

    /// Updates the last activity timestamp.
    fn touch(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.last_activity_at = now;
        self.updated_at = now;
    }

    /// Adds an item to the cart.
    ///
    /// If product already exists, increases quantity.
    pub fn add_item(&mut self, product: &Product, quantity: u32) -> Result<(), CommerceError> {
        if quantity == 0 {
            return Err(CommerceError::InvalidQuantity);
        }

        if !product.status.is_purchasable() {
            return Err(CommerceError::ProductNotAvailable(product.id.0.to_string()));
        }

        // Check if product already in cart
        if let Some(item) = self.items.iter_mut().find(|i| i.product_id == product.id) {
            let new_qty = item.quantity.saturating_add(quantity);

            // Check inventory
            if !product.backorders_allowed && (new_qty as i64) > product.inventory_quantity {
                return Err(CommerceError::InsufficientInventory {
                    product_id: product.id.0.to_string(),
                    available:  product.inventory_quantity as u32,
                    requested:  new_qty,
                });
            }

            item.set_quantity(new_qty);
        } else {
            // Check inventory for new item
            if !product.backorders_allowed && (quantity as i64) > product.inventory_quantity {
                return Err(CommerceError::InsufficientInventory {
                    product_id: product.id.0.to_string(),
                    available:  product.inventory_quantity as u32,
                    requested:  quantity,
                });
            }

            self.items.push(CartItem::from_product(product, quantity));
        }

        self.touch();
        Ok(())
    }

    /// Updates item quantity.
    ///
    /// Removes item if quantity is 0.
    pub fn update_item_quantity(
        &mut self, product_id: &ProductId, quantity: u32,
    ) -> Result<(), CommerceError> {
        if quantity == 0 {
            return self.remove_item(product_id);
        }

        let item = self
            .items
            .iter_mut()
            .find(|i| &i.product_id == product_id)
            .ok_or_else(|| CommerceError::ItemNotInCart(product_id.0.to_string()))?;

        item.set_quantity(quantity);
        self.touch();
        Ok(())
    }

    /// Removes an item from the cart.
    pub fn remove_item(&mut self, product_id: &ProductId) -> Result<(), CommerceError> {
        let initial_len = self.items.len();
        self.items.retain(|i| &i.product_id != product_id);

        if self.items.len() == initial_len {
            return Err(CommerceError::ItemNotInCart(product_id.0.to_string()));
        }

        self.touch();
        Ok(())
    }

    /// Clears all items from the cart.
    pub fn clear(&mut self) {
        self.items.clear();
        self.discounts.clear();
        self.touch();
    }

    /// Applies a discount code.
    pub fn apply_discount(&mut self, discount: AppliedDiscount) -> Result<(), CommerceError> {
        // Check if already applied
        if self.discounts.iter().any(|d| d.code.0 == discount.code.0) {
            return Err(CommerceError::DiscountAlreadyApplied(
                discount.code.0.to_string(),
            ));
        }

        self.discounts.push(discount);
        self.touch();
        Ok(())
    }

    /// Removes a discount code.
    pub fn remove_discount(&mut self, code: &str) -> Result<(), CommerceError> {
        let initial_len = self.discounts.len();
        self.discounts.retain(|d| d.code.0 != code);

        if self.discounts.len() == initial_len {
            return Err(CommerceError::DiscountNotFound(code.to_string()));
        }

        self.touch();
        Ok(())
    }

    /// Sets shipping address.
    pub fn set_shipping_address(&mut self, address: ShippingAddress) {
        self.shipping_address = Some(address);
        self.touch();
    }

    /// Sets billing address.
    pub fn set_billing_address(&mut self, address: ShippingAddress) {
        self.billing_address = Some(address);
        self.touch();
    }

    /// Sets shipping method.
    pub fn set_shipping_method(&mut self, method: ShippingMethod) {
        self.shipping_method = Some(method);
        self.touch();
    }

    /// Calculates cart totals.
    #[must_use]
    pub fn calculate_totals(&self) -> CartTotals {
        CartTotals::calculate(
            &self.items,
            &self.discounts,
            self.shipping_method.as_ref(),
            self.tax_rate,
            self.currency.clone(),
        )
    }

    /// Whether cart has expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            now > expires_at
        } else {
            false
        }
    }

    /// Validates cart is ready for checkout.
    pub fn validate_for_checkout(&self) -> Result<(), CommerceError> {
        if self.is_empty() {
            return Err(CommerceError::CartEmpty);
        }

        if self.status != CartStatus::Active {
            return Err(CommerceError::CartNotActive);
        }

        if self.is_expired() {
            return Err(CommerceError::CartExpired);
        }

        if self.shipping_address.is_none() {
            return Err(CommerceError::ShippingAddressRequired);
        }

        Ok(())
    }
}
