//! # Cart System (GAP-220-D-002)
//!
//! Complete shopping cart management for the e-commerce platform.

use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    errors::CommerceError,
    types::product_catalog::{Currency, Price, Product, ProductId},
};

// ============================================================================
// CORE TYPES
// ============================================================================

/// Unique cart identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CartId(pub Cow<'static, str>);

impl CartId {
    /// Creates a new cart ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(Cow::Owned(id.into()))
    }

    /// Creates a cart ID from a static string slice (zero-copy).
    #[must_use]
    pub fn from_static(id: &'static str) -> Self {
        Self(Cow::Borrowed(id))
    }

    /// Generates a new unique cart ID.
    #[must_use]
    pub fn generate() -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        Self(Cow::Owned(format!("cart-{}", timestamp)))
    }
}

impl std::fmt::Display for CartId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// User/customer identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomerId(pub Cow<'static, str>);

impl CustomerId {
    /// Creates a new customer ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(Cow::Owned(id.into()))
    }

    /// Creates a customer ID from a static string slice (zero-copy).
    #[must_use]
    pub fn from_static(id: &'static str) -> Self {
        Self(Cow::Borrowed(id))
    }

    /// Guest customer ID.
    #[must_use]
    pub fn guest() -> Self {
        Self(Cow::Borrowed("guest"))
    }
}

/// Cart status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CartStatus {
    /// Cart is active and can be modified.
    #[default]
    Active,
    /// Cart has been converted to an order.
    Converted,
    /// Cart was abandoned.
    Abandoned,
    /// Cart has expired.
    Expired,
    /// Cart is merged into another cart.
    Merged,
}

/// Coupon/discount code.
#[derive(Debug, Clone)]
pub struct CouponCode(pub Cow<'static, str>);

impl CouponCode {
    /// Creates a new coupon code.
    #[must_use]
    pub fn new(code: impl Into<String>) -> Self {
        Self(Cow::Owned(code.into().to_uppercase()))
    }

    /// Creates a coupon code from a static string slice (zero-copy).
    #[must_use]
    pub fn from_static(code: &'static str) -> Self {
        Self(Cow::Borrowed(code))
    }
}

/// Type of discount.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscountType {
    /// Percentage discount.
    Percentage,
    /// Fixed amount discount.
    FixedAmount,
    /// Free shipping.
    FreeShipping,
    /// Buy X get Y free.
    BuyXGetY,
}

/// Applied discount on cart.
#[derive(Debug, Clone)]
pub struct AppliedDiscount {
    /// Discount code used.
    pub code:          CouponCode,
    /// Type of discount.
    pub discount_type: DiscountType,
    /// Discount value (percentage or amount).
    pub value:         u64,
    /// Description of the discount.
    pub description:   String,
    /// Amount saved by this discount.
    pub savings:       u64,
}

impl AppliedDiscount {
    /// Creates a percentage discount.
    #[must_use]
    pub fn percentage(code: CouponCode, percent: u64, description: impl Into<String>) -> Self {
        Self {
            code,
            discount_type: DiscountType::Percentage,
            value: percent,
            description: description.into(),
            savings: 0,
        }
    }

    /// Creates a fixed amount discount.
    #[must_use]
    pub fn fixed_amount(code: CouponCode, amount: u64, description: impl Into<String>) -> Self {
        Self {
            code,
            discount_type: DiscountType::FixedAmount,
            value: amount,
            description: description.into(),
            savings: 0,
        }
    }
}

// ============================================================================
// CART ITEM
// ============================================================================

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

// ============================================================================
// SHIPPING
// ============================================================================

/// Shipping address.
#[derive(Debug, Clone, Default)]
pub struct ShippingAddress {
    /// First name.
    pub first_name:    Cow<'static, str>,
    /// Last name.
    pub last_name:     Cow<'static, str>,
    /// Company name.
    pub company:       Option<Cow<'static, str>>,
    /// Address line 1.
    pub address_line1: Cow<'static, str>,
    /// Address line 2.
    pub address_line2: Option<Cow<'static, str>>,
    /// City.
    pub city:          Cow<'static, str>,
    /// State/province.
    pub state:         Cow<'static, str>,
    /// Postal/ZIP code.
    pub postal_code:   Cow<'static, str>,
    /// Country code (ISO 3166-1 alpha-2).
    pub country_code:  Cow<'static, str>,
    /// Phone number.
    pub phone:         Option<Cow<'static, str>>,
}

impl ShippingAddress {
    /// Creates a new shipping address.
    #[must_use]
    pub fn new(
        first_name: impl Into<String>, last_name: impl Into<String>,
        address_line1: impl Into<String>, city: impl Into<String>, state: impl Into<String>,
        postal_code: impl Into<String>, country_code: impl Into<String>,
    ) -> Self {
        Self {
            first_name:    Cow::Owned(first_name.into()),
            last_name:     Cow::Owned(last_name.into()),
            company:       None,
            address_line1: Cow::Owned(address_line1.into()),
            address_line2: None,
            city:          Cow::Owned(city.into()),
            state:         Cow::Owned(state.into()),
            postal_code:   Cow::Owned(postal_code.into()),
            country_code:  Cow::Owned(country_code.into()),
            phone:         None,
        }
    }

    /// Full name.
    #[must_use]
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

/// Shipping method.
#[derive(Debug, Clone)]
pub struct ShippingMethod {
    /// Method identifier.
    pub id:                 Cow<'static, str>,
    /// Display name.
    pub name:               Cow<'static, str>,
    /// Description.
    pub description:        Cow<'static, str>,
    /// Shipping cost.
    pub cost:               Price,
    /// Estimated delivery days (min).
    pub estimated_days_min: u32,
    /// Estimated delivery days (max).
    pub estimated_days_max: u32,
    /// Whether tracking is available.
    pub has_tracking:       bool,
}

impl ShippingMethod {
    /// Creates a new shipping method.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>, cost: Price) -> Self {
        Self {
            id: Cow::Owned(id.into()),
            name: Cow::Owned(name.into()),
            description: Cow::Owned(String::new()),
            cost,
            estimated_days_min: 3,
            estimated_days_max: 7,
            has_tracking: true,
        }
    }

    /// Creates a shipping method from static strings (zero-copy).
    #[must_use]
    pub fn from_static(id: &'static str, name: &'static str, cost: Price) -> Self {
        Self {
            id: Cow::Borrowed(id),
            name: Cow::Borrowed(name),
            description: Cow::Borrowed(""),
            cost,
            estimated_days_min: 3,
            estimated_days_max: 7,
            has_tracking: true,
        }
    }

    /// Free shipping method.
    #[must_use]
    pub fn free_shipping() -> Self {
        Self {
            id:                 Cow::Borrowed("free"),
            name:               Cow::Borrowed("Free Shipping"),
            description:        Cow::Borrowed("Standard free shipping"),
            cost:               Price::new(0, Currency::usd(), 2),
            estimated_days_min: 5,
            estimated_days_max: 10,
            has_tracking:       false,
        }
    }

    /// Estimated delivery range string.
    #[must_use]
    pub fn delivery_estimate(&self) -> String {
        if self.estimated_days_min == self.estimated_days_max {
            format!("{} business days", self.estimated_days_min)
        } else {
            format!(
                "{}-{} business days",
                self.estimated_days_min, self.estimated_days_max
            )
        }
    }
}

// ============================================================================
// CART TOTALS
// ============================================================================

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

// ============================================================================
// SHOPPING CART
// ============================================================================

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

// ============================================================================
// CART SERVICE
// ============================================================================

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

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::product_catalog::{Product, ProductId, ProductStatus, Sku};

    fn create_test_product(id: &str, price: u64) -> Product {
        let mut product = Product::new(
            ProductId::new(id),
            Sku::new(format!("SKU-{}", id)),
            format!("Product {}", id),
        );
        product.status = ProductStatus::Active;
        product.price = Price::new(price, Currency::usd(), 2);
        product.inventory_quantity = 100;
        product
    }

    #[test]
    fn test_cart_creation() {
        let cart = Cart::new(CustomerId::new("customer-1"));

        assert!(cart.is_empty());
        assert_eq!(cart.status, CartStatus::Active);
        assert!(!cart.is_expired());
    }

    #[test]
    fn test_add_item() {
        let mut cart = Cart::new(CustomerId::new("customer-1"));
        let product = create_test_product("001", 1000);

        cart.add_item(&product, 2).expect("should add item");

        assert!(!cart.is_empty());
        assert_eq!(cart.total_quantity(), 2);
        assert_eq!(cart.unique_item_count(), 1);
    }

    #[test]
    fn test_add_same_item_increases_quantity() {
        let mut cart = Cart::new(CustomerId::new("customer-1"));
        let product = create_test_product("001", 1000);

        cart.add_item(&product, 2).expect("add first");
        cart.add_item(&product, 3).expect("add second");

        assert_eq!(cart.unique_item_count(), 1);
        assert_eq!(cart.total_quantity(), 5);
    }

    #[test]
    fn test_remove_item() {
        let mut cart = Cart::new(CustomerId::new("customer-1"));
        let product = create_test_product("001", 1000);

        cart.add_item(&product, 2).expect("add");
        cart.remove_item(&product.id).expect("remove");

        assert!(cart.is_empty());
    }

    #[test]
    fn test_update_quantity() {
        let mut cart = Cart::new(CustomerId::new("customer-1"));
        let product = create_test_product("001", 1000);

        cart.add_item(&product, 2).expect("add");
        cart.update_item_quantity(&product.id, 5).expect("update");

        assert_eq!(cart.total_quantity(), 5);
    }

    #[test]
    fn test_calculate_totals() {
        let mut cart = Cart::new(CustomerId::new("customer-1"));
        cart.tax_rate = 10.0;

        let product1 = create_test_product("001", 1000);
        let product2 = create_test_product("002", 2000);

        cart.add_item(&product1, 2).expect("add 1");
        cart.add_item(&product2, 1).expect("add 2");

        let totals = cart.calculate_totals();

        assert_eq!(totals.subtotal, 4000); // (1000*2) + (2000*1)
        assert_eq!(totals.tax_total, 400); // 10% of 4000
        assert_eq!(totals.item_count, 3);
    }

    #[test]
    fn test_apply_discount() {
        let mut cart = Cart::new(CustomerId::new("customer-1"));
        let discount = AppliedDiscount::percentage(CouponCode::new("SAVE10"), 10, "10% off");

        cart.apply_discount(discount).expect("apply discount");
        assert_eq!(cart.discounts.len(), 1);
    }

    #[test]
    fn test_duplicate_discount_rejected() {
        let mut cart = Cart::new(CustomerId::new("customer-1"));
        let discount1 = AppliedDiscount::percentage(CouponCode::new("SAVE10"), 10, "10% off");
        let discount2 =
            AppliedDiscount::percentage(CouponCode::new("SAVE10"), 10, "Another 10% off");

        cart.apply_discount(discount1).expect("first");
        let result = cart.apply_discount(discount2);
        assert!(result.is_err());
    }

    #[test]
    fn test_cart_service() {
        let service = CartService::new();
        let customer_id = CustomerId::new("customer-1");

        let cart = service.create_cart(customer_id.clone()).expect("create");
        let retrieved = service.get_cart(&cart.id).expect("get");

        assert_eq!(cart.id, retrieved.id);
    }

    #[test]
    fn test_validate_for_checkout() {
        let mut cart = Cart::new(CustomerId::new("customer-1"));
        let product = create_test_product("001", 1000);

        // Empty cart fails
        assert!(cart.validate_for_checkout().is_err());

        // No shipping address fails
        cart.add_item(&product, 1).expect("add");
        assert!(cart.validate_for_checkout().is_err());

        // With shipping address succeeds
        cart.set_shipping_address(ShippingAddress::new(
            "John",
            "Doe",
            "123 Main St",
            "City",
            "State",
            "12345",
            "US",
        ));
        assert!(cart.validate_for_checkout().is_ok());
    }
}
