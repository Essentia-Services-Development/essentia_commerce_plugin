//! # Product Catalog Tests (GAP-220-D-001)
//!
//! Test suite for product catalog functionality.

#[cfg(test)]
mod tests {
    use crate::implementation::product_catalog::service::ProductCatalog;
    use crate::types::product_catalog::*;

    #[test]
    fn test_product_creation() {
        let product = Product::new(
            ProductId::new("prod-001"),
            Sku::new("SKU-001"),
            "Test Product",
        );

        assert_eq!(product.id.as_str(), "prod-001");
        assert_eq!(product.name, "Test Product");
        assert_eq!(product.status, ProductStatus::Draft);
    }

    #[test]
    fn test_price_operations() {
        let price1 = Price::new(1000, Currency::usd(), 2);
        let price2 = Price::new(500, Currency::usd(), 2);

        let total = price1.add(&price2).expect("should add prices");
        assert_eq!(total.amount, 1500);
        assert_eq!(total.display_amount(), 15.0);
    }

    #[test]
    fn test_catalog_add_product() {
        let catalog = ProductCatalog::new();
        let product = Product::new(
            ProductId::new("prod-001"),
            Sku::new("SKU-001"),
            "Test Product",
        );

        catalog.add_product(product).expect("should add product");

        let retrieved =
            catalog.get_product(&ProductId::new("prod-001")).expect("should get product");
        assert_eq!(retrieved.name, "Test Product");
    }

    #[test]
    fn test_catalog_duplicate_sku() {
        let catalog = ProductCatalog::new();

        let product1 = Product::new(ProductId::new("prod-001"), Sku::new("SKU-001"), "Product 1");
        let product2 = Product::new(
            ProductId::new("prod-002"),
            Sku::new("SKU-001"), // Same SKU
            "Product 2",
        );

        catalog.add_product(product1).expect("should add first product");
        let result = catalog.add_product(product2);
        assert!(result.is_err());
    }

    #[test]
    fn test_category_hierarchy() {
        let catalog = ProductCatalog::new();

        let root = Category::new(CategoryId::new("cat-root"), "Electronics");
        let child = Category::new(CategoryId::new("cat-phones"), "Phones")
            .with_parent(CategoryId::new("cat-root"));

        catalog.add_category(root).expect("should add root");
        catalog.add_category(child).expect("should add child");

        let children = catalog
            .get_child_categories(&CategoryId::new("cat-root"))
            .expect("should get children");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "Phones");
    }

    #[test]
    fn test_product_search() {
        let catalog = ProductCatalog::new();

        let mut product1 =
            Product::new(ProductId::new("prod-001"), Sku::new("SKU-001"), "iPhone 15");
        product1.status = ProductStatus::Active;
        product1.price = Price::new(99900, crate::types::product_catalog::Currency::usd(), 2);

        let mut product2 = Product::new(
            ProductId::new("prod-002"),
            Sku::new("SKU-002"),
            "Samsung Galaxy",
        );
        product2.status = ProductStatus::Active;
        product2.price = Price::new(79900, crate::types::product_catalog::Currency::usd(), 2);

        catalog.add_product(product1).expect("add product1");
        catalog.add_product(product2).expect("add product2");

        let filter = ProductFilter::new().with_status(ProductStatus::Active);
        let results = catalog
            .search_products(&filter, ProductSortOrder::PriceAsc, 0, 10)
            .expect("search should succeed");

        assert_eq!(results.total_count, 2);
        assert_eq!(results.products[0].name, "Samsung Galaxy"); // Lower price first
    }

    #[test]
    fn test_effective_price() {
        let mut product = Product::new(
            ProductId::new("prod-001"),
            Sku::new("SKU-001"),
            "Test Product",
        );
        product.price = Price::new(10000, crate::types::product_catalog::Currency::usd(), 2);

        assert_eq!(product.effective_price().amount, 10000);
        assert!(!product.is_on_sale());

        product.sale_price = Some(Price::new(7500, crate::types::product_catalog::Currency::usd(), 2));
        assert_eq!(product.effective_price().amount, 7500);
        assert!(product.is_on_sale());
    }

    #[test]
    fn test_product_status() {
        assert!(ProductStatus::Active.is_visible());
        assert!(ProductStatus::Active.is_purchasable());
        assert!(ProductStatus::OutOfStock.is_visible());
        assert!(!ProductStatus::OutOfStock.is_purchasable());
        assert!(!ProductStatus::Draft.is_visible());
    }
}
