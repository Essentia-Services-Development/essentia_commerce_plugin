//! Shipping address and method types

use std::borrow::Cow;

use crate::types::product_catalog::{Currency, Price};

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
