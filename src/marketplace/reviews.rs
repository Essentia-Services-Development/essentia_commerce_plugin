//! # Marketplace Review and Rating Types
//!
//! Types and structures for reviews, ratings, and seller profiles.

/// A review for a listing or seller
#[derive(Debug, Clone)]
pub struct Review {
    /// Unique review ID
    pub id: ReviewId,
    /// Order this review is for
    pub order_id: super::orders::OrderId,
    /// Listing reviewed
    pub listing_id: super::ListingId,
    /// Reviewer (buyer)
    pub reviewer: String, // Placeholder for PeerNodeId
    /// Seller being reviewed
    pub seller: String, // Placeholder for PeerNodeId
    /// Overall rating (1-5)
    pub rating: u8,
    /// Category ratings
    pub category_ratings: CategoryRatings,
    /// Review text
    pub text: String,
    /// Pros mentioned
    pub pros: Vec<String>,
    /// Cons mentioned
    pub cons: Vec<String>,
    /// Review timestamp
    pub created_at: u64,
    /// Was this a verified purchase?
    pub verified_purchase: bool,
    /// Helpful votes
    pub helpful_count: u32,
    /// Seller response
    pub seller_response: Option<SellerResponse>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReviewId(String);

impl ReviewId {
    pub fn new() -> Self {
        Self(format!("review-{}", essentia_uuid::Uuid::new_v4()))
    }
}

#[derive(Debug, Clone)]
pub struct CategoryRatings {
    /// Quality of product/service
    pub quality: u8,
    /// Value for money
    pub value: u8,
    /// Communication (for services)
    pub communication: Option<u8>,
    /// Timeliness (for services)
    pub timeliness: Option<u8>,
    /// Documentation quality
    pub documentation: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct SellerResponse {
    pub text: String,
    pub responded_at: u64,
}

/// Seller profile and reputation
#[derive(Debug, Clone)]
pub struct SellerProfile {
    /// Seller node ID
    pub node_id: String, // Placeholder for PeerNodeId
    /// Display name
    pub display_name: String,
    /// Bio
    pub bio: String,
    /// Avatar hash
    pub avatar_hash: Option<String>,
    /// Joined timestamp
    pub joined_at: u64,
    /// VCS reputation (from CR-108-F3)
    pub vcs_reputation: u32,
    /// Marketplace reputation
    pub marketplace_reputation: SellerReputation,
    /// Active listings count
    pub active_listings: u32,
    /// Completed orders count
    pub completed_orders: u32,
    /// Verification level
    pub verification: VerificationLevel,
    /// Specializations
    pub specializations: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SellerReputation {
    /// Total reviews
    pub review_count: u32,
    /// Average rating
    pub average_rating: f32,
    /// Rating distribution [1-star, 2-star, ..., 5-star]
    pub rating_distribution: [u32; 5],
    /// Success rate percentage
    pub success_rate: f64,
    /// Response time in hours
    pub avg_response_time_hours: f32,
    /// Total sales volume in sats
    pub total_sales_sats: u64,
    /// Last activity timestamp
    pub last_active: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationLevel {
    /// Unverified
    None,
    /// Email verified
    Email,
    /// Identity verified
    Identity,
    /// Business verified
    Business,
    /// Premium verification
    Premium,
}

/// Review summary for listings
#[derive(Debug, Clone)]
pub struct ReviewSummary {
    /// Listing ID
    pub listing_id: super::ListingId,
    /// Total reviews
    pub total_reviews: u32,
    /// Average rating
    pub average_rating: f32,
    /// Rating distribution
    pub rating_distribution: [u32; 5],
    /// Recent reviews (last 30 days)
    pub recent_reviews: u32,
    /// Verified reviews only
    pub verified_only: bool,
}

/// Review filter for queries
#[derive(Debug, Clone)]
pub struct ReviewFilter {
    /// Minimum rating
    pub min_rating: Option<u8>,
    /// Verified purchases only
    pub verified_only: bool,
    /// Has seller response
    pub has_response: Option<bool>,
    /// Date range
    pub date_range: Option<(u64, u64)>,
    /// Sort order
    pub sort_by: ReviewSort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewSort {
    /// Newest first
    Newest,
    /// Oldest first
    Oldest,
    /// Highest rated
    HighestRated,
    /// Lowest rated
    LowestRated,
    /// Most helpful
    MostHelpful,
}
