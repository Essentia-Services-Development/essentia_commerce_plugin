//! # Marketplace Search Index
//!
//! Full-text search and filtering for marketplace listings.

use std::collections::{HashMap, HashSet};

use crate::errors::MarketplaceError;

/// Search index result type
pub type SearchResult<T> = Result<T, MarketplaceError>;

/// Full-text search index for marketplace
pub struct SearchIndex {
    /// Full-text search index (term -> listing IDs)
    full_text:         HashMap<String, HashSet<super::ListingId>>,
    /// Tag-based search index
    tags:              HashMap<String, HashSet<super::ListingId>>,
    /// Seller listings index
    seller_listings:   HashMap<String, HashSet<super::ListingId>>,
    /// Category index
    category_listings: HashMap<super::ListingCategory, HashSet<super::ListingId>>,
    /// Price range index (simplified)
    price_ranges:      HashMap<String, HashSet<super::ListingId>>,
    /// Rating index
    rating_listings:   HashMap<u8, HashSet<super::ListingId>>,
}

impl SearchIndex {
    /// Create a new search index
    pub fn new() -> SearchResult<Self> {
        Ok(Self {
            full_text:         HashMap::new(),
            tags:              HashMap::new(),
            seller_listings:   HashMap::new(),
            category_listings: HashMap::new(),
            price_ranges:      HashMap::new(),
            rating_listings:   HashMap::new(),
        })
    }

    /// Index a new listing
    pub fn index_listing(&mut self, listing: &super::MarketplaceListing) -> SearchResult<()> {
        let listing_id = &listing.id;

        // Index full-text search terms
        self.index_full_text(listing_id, &listing.title);
        self.index_full_text(listing_id, &listing.description);
        self.index_full_text(listing_id, &listing.short_description);

        // Index tags
        for tag in &listing.tags {
            self.tags.entry(tag.clone()).or_default().insert(listing_id.clone());
        }

        // Index seller
        self.seller_listings
            .entry(listing.seller.clone())
            .or_default()
            .insert(listing_id.clone());

        // Index category
        self.category_listings
            .entry(listing.category)
            .or_default()
            .insert(listing_id.clone());

        // Index price range (simplified bucketing)
        let price_bucket = self.get_price_bucket(listing);
        self.price_ranges.entry(price_bucket).or_default().insert(listing_id.clone());

        Ok(())
    }

    /// Remove a listing from the index
    pub fn remove_listing(&mut self, listing_id: &super::ListingId) -> SearchResult<()> {
        // Remove from all indices (simplified - would need full listing data for
        // complete removal)
        for ids in self.full_text.values_mut() {
            ids.remove(listing_id);
        }
        for ids in self.tags.values_mut() {
            ids.remove(listing_id);
        }
        for ids in self.seller_listings.values_mut() {
            ids.remove(listing_id);
        }
        for ids in self.category_listings.values_mut() {
            ids.remove(listing_id);
        }
        for ids in self.price_ranges.values_mut() {
            ids.remove(listing_id);
        }
        for ids in self.rating_listings.values_mut() {
            ids.remove(listing_id);
        }
        Ok(())
    }

    /// Search listings
    pub fn search(
        &self, query: &str, filters: &super::SearchFilters,
    ) -> SearchResult<Vec<super::ListingId>> {
        let mut candidates = HashSet::new();

        // Full-text search
        if !query.is_empty() {
            let query_terms: Vec<&str> = query.split_whitespace().collect();
            for term in query_terms {
                if let Some(ids) = self.full_text.get(&term.to_lowercase()) {
                    if candidates.is_empty() {
                        candidates.extend(ids.iter().cloned());
                    } else {
                        candidates.retain(|id| ids.contains(id));
                    }
                } else if candidates.is_empty() {
                    // No matches for this term and no previous candidates
                    return Ok(Vec::new());
                }
            }
        }

        // Apply category filter
        if let Some(category) = filters.category {
            if let Some(cat_ids) = self.category_listings.get(&category) {
                if candidates.is_empty() {
                    candidates.extend(cat_ids.iter().cloned());
                } else {
                    candidates.retain(|id| cat_ids.contains(id));
                }
            } else if candidates.is_empty() {
                return Ok(Vec::new());
            }
        }

        // Apply price range filter
        if let Some((min_price, max_price)) = filters.price_range {
            let price_bucket = self.get_price_bucket_from_range(min_price, max_price);
            if let Some(price_ids) = self.price_ranges.get(&price_bucket) {
                if candidates.is_empty() {
                    candidates.extend(price_ids.iter().cloned());
                } else {
                    candidates.retain(|id| price_ids.contains(id));
                }
            } else if candidates.is_empty() {
                return Ok(Vec::new());
            }
        }

        // Convert to sorted vec (by relevance - simplified)
        let mut results: Vec<_> = candidates.into_iter().collect();
        results.sort_by(|a, b| a.0.cmp(&b.0)); // Simple ID-based sorting

        Ok(results)
    }

    /// Index full-text terms
    fn index_full_text(&mut self, listing_id: &super::ListingId, text: &str) {
        let terms = self.tokenize(text);
        for term in terms {
            self.full_text.entry(term).or_default().insert(listing_id.clone());
        }
    }

    /// Simple tokenization (lowercase, remove punctuation)
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get price bucket for a listing
    fn get_price_bucket(&self, listing: &super::MarketplaceListing) -> String {
        match &listing.pricing {
            super::PricingModel::OneTime { price_sats } => self.price_bucket(*price_sats),
            super::PricingModel::Subscription { price_sats, .. } => self.price_bucket(*price_sats),
            super::PricingModel::PayWhatYouWant { minimum_sats, .. } => {
                self.price_bucket(*minimum_sats)
            },
            super::PricingModel::Free => "free".to_string(),
            super::PricingModel::Hourly { rate_sats, .. } => {
                format!("hourly_{}", self.price_bucket(*rate_sats))
            },
            super::PricingModel::FixedProject { price_sats, .. } => {
                format!("project_{}", self.price_bucket(*price_sats))
            },
        }
    }

    /// Get price bucket from range
    fn get_price_bucket_from_range(&self, min_price: u64, max_price: u64) -> String {
        if min_price == 0 && max_price == 0 {
            "free".to_string()
        } else {
            format!(
                "{}_{}",
                self.price_bucket(min_price),
                self.price_bucket(max_price)
            )
        }
    }

    /// Convert price to bucket
    fn price_bucket(&self, price_sats: u64) -> String {
        match price_sats {
            0 => "free".to_string(),
            1..=1000 => "micro".to_string(),
            1001..=10000 => "small".to_string(),
            10001..=100000 => "medium".to_string(),
            100001..=1000000 => "large".to_string(),
            _ => "premium".to_string(),
        }
    }
}
