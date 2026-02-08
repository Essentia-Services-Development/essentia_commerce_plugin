//! Core type definitions for the cart system

use std::borrow::Cow;

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
