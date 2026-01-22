// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use crate::types::{
        inventory_sync::{InventoryLocation, InventoryService, LocationId},
        product_catalog::ProductId,
    };

    #[test]
    fn test_inventory_service_creation() {
        let service = InventoryService::new();

        // Default warehouse should exist
        let location = service.get_location(&LocationId::default_warehouse());
        assert!(location.is_ok());
    }

    #[test]
    fn test_set_and_get_inventory() {
        let service = InventoryService::new();
        let product_id = ProductId::new("prod-001");
        let location_id = LocationId::default_warehouse();

        service
            .set_inventory(
                product_id.clone(),
                location_id.clone(),
                100,
                "Initial stock",
            )
            .expect("set inventory");

        let level = service.get_inventory(&product_id, &location_id).expect("get");

        assert_eq!(level.on_hand, 100);
        assert_eq!(level.available, 100);
    }

    #[test]
    fn test_reserve_stock() {
        let service = InventoryService::new();
        let product_id = ProductId::new("prod-001");
        let location_id = LocationId::default_warehouse();

        service
            .set_inventory(product_id.clone(), location_id.clone(), 100, "Initial")
            .expect("set");

        service
            .reserve_stock(&product_id, &location_id, 30, "ORD-001")
            .expect("reserve");

        let level = service.get_inventory(&product_id, &location_id).expect("get");
        assert_eq!(level.on_hand, 100);
        assert_eq!(level.committed, 30);
        assert_eq!(level.available, 70);
    }

    #[test]
    fn test_reserve_insufficient_stock() {
        let service = InventoryService::new();
        let product_id = ProductId::new("prod-001");
        let location_id = LocationId::default_warehouse();

        service
            .set_inventory(product_id.clone(), location_id.clone(), 10, "Low stock")
            .expect("set");

        let result = service.reserve_stock(&product_id, &location_id, 50, "ORD-001");
        assert!(result.is_err());
    }

    #[test]
    fn test_commit_stock() {
        let service = InventoryService::new();
        let product_id = ProductId::new("prod-001");
        let location_id = LocationId::default_warehouse();

        service
            .set_inventory(product_id.clone(), location_id.clone(), 100, "Initial")
            .expect("set");

        service
            .reserve_stock(&product_id, &location_id, 30, "ORD-001")
            .expect("reserve");
        service.commit_stock(&product_id, &location_id, 30, "ORD-001").expect("commit");

        let level = service.get_inventory(&product_id, &location_id).expect("get");
        assert_eq!(level.on_hand, 70);
        assert_eq!(level.committed, 0);
        assert_eq!(level.available, 70);
    }

    #[test]
    fn test_receive_stock() {
        let service = InventoryService::new();
        let product_id = ProductId::new("prod-001");
        let location_id = LocationId::default_warehouse();

        service
            .set_inventory(product_id.clone(), location_id.clone(), 50, "Initial")
            .expect("set");

        service
            .receive_stock(&product_id, &location_id, 100, "PO-001")
            .expect("receive");

        let level = service.get_inventory(&product_id, &location_id).expect("get");
        assert_eq!(level.on_hand, 150);
        assert_eq!(level.available, 150);
    }

    #[test]
    fn test_low_stock_detection() {
        let service = InventoryService::new();
        let product_id = ProductId::new("prod-001");
        let location_id = LocationId::default_warehouse();

        service
            .set_inventory(product_id.clone(), location_id.clone(), 5, "Low stock")
            .expect("set");

        let level = service.get_inventory(&product_id, &location_id).expect("get");
        assert!(level.is_low_stock());

        let low_stock = service.get_low_stock_products().expect("get low");
        assert_eq!(low_stock.len(), 1);
    }

    #[test]
    fn test_total_available_across_locations() {
        let service = InventoryService::new();
        let product_id = ProductId::new("prod-001");

        let location1 = LocationId::default_warehouse();
        let location2 = LocationId::new("warehouse-secondary");

        service
            .add_location(InventoryLocation::warehouse(
                location2.clone(),
                "Secondary Warehouse",
            ))
            .expect("add location");

        service
            .set_inventory(product_id.clone(), location1, 100, "Stock 1")
            .expect("set 1");
        service
            .set_inventory(product_id.clone(), location2, 50, "Stock 2")
            .expect("set 2");

        let total = service.get_total_available(&product_id).expect("total");
        assert_eq!(total, 150);
    }

    #[test]
    fn test_adjustment_history() {
        let service = InventoryService::new();
        let product_id = ProductId::new("prod-001");
        let location_id = LocationId::default_warehouse();

        service
            .set_inventory(product_id.clone(), location_id.clone(), 100, "Initial")
            .expect("set");
        service.receive_stock(&product_id, &location_id, 50, "PO-001").expect("receive");
        service
            .reserve_stock(&product_id, &location_id, 30, "ORD-001")
            .expect("reserve");

        let history = service.get_adjustment_history(&product_id, None).expect("history");
        assert_eq!(history.len(), 3);
    }
}
