//! # Marketplace Integration Tests
//!
//! End-to-end tests for the decentralized marketplace functionality.

use crate::marketplace::*;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_listing_id_creation() {
        let id1 = ListingId::new();
        let id2 = ListingId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_listing_id_from_content_hash() {
        let hash = "test_content_hash";
        let id = ListingId::from_content_hash(hash);
        assert!(id.0.contains(hash));
    }

    #[test]
    fn test_search_index_creation() {
        let search_index = search::SearchIndex::new();
        assert!(search_index.is_ok());
    }

    #[test]
    fn test_escrow_manager_creation() {
        let escrow_manager = escrow::EscrowManager::new();
        assert!(escrow_manager.is_ok());
    }

    #[test]
    fn test_content_delivery_service_creation() {
        let _delivery_service = delivery::ContentDeliveryService::new();
        // Test passes if it doesn't panic
    }

    #[test]
    fn test_p2p_sync_creation() {
        let sync_service = sync::P2PCatalogSync::new();
        assert!(sync_service.is_ok());
    }

    #[test]
    fn test_order_id_creation() {
        let id1 = orders::OrderId::new();
        let id2 = orders::OrderId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_review_id_creation() {
        let id1 = reviews::ReviewId::new();
        let id2 = reviews::ReviewId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_escrow_id_creation() {
        let id1 = escrow::EscrowId::new();
        let id2 = escrow::EscrowId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_content_hash_creation() {
        let hash = delivery::ContentHash::new("test_hash".to_string());
        assert_eq!(hash.as_str(), "test_hash");
    }
}
