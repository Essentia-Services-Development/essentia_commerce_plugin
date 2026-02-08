//! # Cart System (GAP-220-D-002)
//!
//! Complete shopping cart management for the e-commerce platform.

mod cart;
mod item;
mod service;
mod shipping;
mod types;

pub use cart::{Cart, CartTotals};
pub use item::CartItem;
pub use service::CartService;
pub use shipping::{ShippingAddress, ShippingMethod};
pub use types::{AppliedDiscount, CartId, CartStatus, CouponCode, CustomerId, DiscountType};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::product_catalog::{Currency, Price, Product, ProductId, ProductStatus, Sku};

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
