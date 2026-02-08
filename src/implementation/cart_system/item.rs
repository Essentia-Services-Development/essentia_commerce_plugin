//! Cart item type definition

use std::{borrow::Cow, collections::HashMap};

use crate::types::product_catalog::{Price, Product, ProductId};

use super::types::AppliedDiscount;

/// Item in the shopping cart.
#[derive(Debug, Clone)]
pub struct CartItem {
    /// Product ID.
    pub product_id:     ProductId,
    /// Variant ID (if applicable).
    pub variant_id:     Option<ProductId>,
    /// Product name (cached for display).
    pub product_name:   Cow<'static, str>,
    /// Product SKU (cached).
    pub product_sku:    Cow<'static, str>,
    /// Product image URL (cached).
    pub image_url:      Option<Cow<'static, str>>,
    /// Quantity.
    pub quantity:       u32,
    /// Unit price at time of adding.
    pub unit_price:     Price,
    /// Original price (before any sale).
    pub original_price: Price,
    /// Applied item-level discounts.
    pub discounts:      Vec<AppliedDiscount>,
    /// Custom options selected.
    pub custom_options: HashMap<Cow<'static, str>, Cow<'static, str>>,
    /// When item was added.
    pub added_at:       u64,
    /// When item was last updated.
    pub updated_at:     u64,
}

impl CartItem {
    /// Creates a new cart item from a product.
    #[must_use]
    pub fn from_product(product: &Product, quantity: u32) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            product_id: product.id.clone(),
            variant_id: None,
            product_name: Cow::Owned(product.name.clone()),
            product_sku: Cow::Owned(product.sku.0.to_string()),
            image_url: product.primary_image().map(|img| Cow::Owned(img.url.clone())),
            quantity,
            unit_price: product.effective_price().clone(),
            original_price: product.price.clone(),
            discounts: Vec::new(),
            custom_options: HashMap::new(),
            added_at: now,
            updated_at: now,
        }
    }

    /// Calculates line total before discounts.
    #[must_use]
    pub fn subtotal(&self) -> u64 {
        self.unit_price.amount * u64::from(self.quantity)
    }

    /// Calculates total discounts for this item.
    #[must_use]
    pub fn total_discount(&self) -> u64 {
        self.discounts.iter().map(|d| d.savings).sum()
    }

    /// Calculates line total after discounts.
    #[must_use]
    pub fn total(&self) -> u64 {
        self.subtotal().saturating_sub(self.total_discount())
    }

    /// Whether item is on sale.
    #[must_use]
    pub fn is_on_sale(&self) -> bool {
        self.unit_price.amount < self.original_price.amount
    }

    /// Calculates savings from sale price.
    #[must_use]
    pub fn sale_savings(&self) -> u64 {
        if self.is_on_sale() {
            (self.original_price.amount - self.unit_price.amount) * u64::from(self.quantity)
        } else {
            0
        }
    }

    /// Updates quantity.
    pub fn set_quantity(&mut self, quantity: u32) {
        self.quantity = quantity;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }
}
