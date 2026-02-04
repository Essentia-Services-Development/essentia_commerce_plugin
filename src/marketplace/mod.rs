//! # Decentralized Marketplace Types
//!
//! Types and structures for the P2P decentralized marketplace integration
//! with VCS plugins, content, and services.

pub mod delivery;
pub mod escrow;
pub mod orders;
pub mod reviews;
pub mod search;
pub mod service;
pub mod sync;

#[cfg(test)]
mod tests;

use std::fmt::Debug;

/// Unique listing identifier (content-addressed)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListingId(String);

impl ListingId {
    /// Create from content hash
    pub fn from_content_hash(hash: &str) -> Self {
        Self(format!("listing-{hash}"))
    }

    pub fn new() -> Self {
        Self(format!("listing-{}", essentia_uuid::Uuid::new_v4()))
    }
}

/// Category of marketplace listing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ListingCategory {
    // Software Products
    Plugin,
    Extension,
    Theme,
    Template,
    Library,
    Framework,

    // Content
    Tutorial,
    Course,
    Documentation,
    EBook,

    // Services
    CodeReview,
    Testing,
    Consulting,
    Freelance,
    Mentoring,

    // Licenses
    RepositoryLicense,
    ComponentLicense,
    SaaSAccess,
}

impl ListingCategory {
    /// Get category display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Plugin => "VCS Plugin",
            Self::Extension => "Editor Extension",
            Self::Theme => "Theme",
            Self::Template => "Code Template",
            Self::Library => "Library",
            Self::Framework => "Framework",
            Self::Tutorial => "Tutorial",
            Self::Course => "Course",
            Self::Documentation => "Documentation",
            Self::EBook => "E-Book",
            Self::CodeReview => "Code Review Service",
            Self::Testing => "Testing Service",
            Self::Consulting => "Consulting",
            Self::Freelance => "Freelance Work",
            Self::Mentoring => "Mentoring",
            Self::RepositoryLicense => "Repository License",
            Self::ComponentLicense => "Component License",
            Self::SaaSAccess => "SaaS Access",
        }
    }

    /// Is this a digital product (instant delivery)?
    pub fn is_digital_product(&self) -> bool {
        matches!(
            self,
            Self::Plugin
                | Self::Extension
                | Self::Theme
                | Self::Template
                | Self::Library
                | Self::Framework
                | Self::Tutorial
                | Self::Course
                | Self::Documentation
                | Self::EBook
        )
    }

    /// Is this a service (requires fulfillment)?
    pub fn is_service(&self) -> bool {
        matches!(
            self,
            Self::CodeReview | Self::Testing | Self::Consulting | Self::Freelance | Self::Mentoring
        )
    }
}

/// Pricing model for a listing
#[derive(Debug, Clone)]
pub enum PricingModel {
    /// One-time purchase
    OneTime { price_sats: u64 },
    /// Subscription (recurring)
    Subscription { price_sats: u64, interval: SubscriptionInterval },
    /// Pay what you want
    PayWhatYouWant { minimum_sats: u64, suggested_sats: u64 },
    /// Free
    Free,
    /// Hourly rate (for services)
    Hourly { rate_sats: u64, minimum_hours: u32 },
    /// Fixed project price
    FixedProject { price_sats: u64, milestones: Vec<Milestone> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscriptionInterval {
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone)]
pub struct Milestone {
    pub name:         String,
    pub description:  String,
    pub percentage:   u8, // Percentage of total price
    pub deliverables: Vec<String>,
}

/// A marketplace listing
#[derive(Debug, Clone)]
pub struct MarketplaceListing {
    /// Unique listing ID
    pub id:                ListingId,
    /// Seller node
    pub seller:            String, // Placeholder for PeerNodeId
    /// Category
    pub category:          ListingCategory,
    /// Title
    pub title:             String,
    /// Description (Markdown)
    pub description:       String,
    /// Short description for previews
    pub short_description: String,
    /// Pricing
    pub pricing:           PricingModel,
    /// Tags for searchability
    pub tags:              Vec<String>,
    /// Preview images/screenshots
    pub previews:          Vec<PreviewAsset>,
    /// Version (for digital products)
    pub version:           Option<String>,
    /// Repository reference (if applicable)
    pub repo_id:           Option<String>, // Placeholder for RepoId
    /// License type
    pub license:           LicenseType,
    /// Creation timestamp
    pub created_at:        u64,
    /// Last updated
    pub updated_at:        u64,
    /// Status
    pub status:            ListingStatus,
    /// Statistics
    pub stats:             ListingStats,
    /// Requirements (for services)
    pub requirements:      Option<ServiceRequirements>,
}

#[derive(Debug, Clone)]
pub struct PreviewAsset {
    pub asset_type: AssetType,
    pub url:        String,
    pub hash:       String,
    pub alt_text:   String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    Image,
    Video,
    Demo,
    Documentation,
}

#[derive(Debug, Clone)]
pub enum LicenseType {
    /// Open source (MIT, Apache, etc.)
    OpenSource { spdx_id: String },
    /// Proprietary with usage rights
    Proprietary { terms_hash: String },
    /// Creative Commons
    CreativeCommons { cc_type: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListingStatus {
    /// Draft - not published
    Draft,
    /// Active - published and available
    Active,
    /// Paused - temporarily unavailable
    Paused,
    /// Discontinued - no longer available
    Discontinued,
}

#[derive(Debug, Clone)]
pub struct ListingStats {
    /// Number of views
    pub views:        u64,
    /// Number of purchases
    pub purchases:    u64,
    /// Average rating (0.0 to 5.0)
    pub avg_rating:   f64,
    /// Number of reviews
    pub review_count: u32,
    /// Revenue generated (in sats)
    pub revenue_sats: u64,
}

#[derive(Debug, Clone)]
pub struct ServiceRequirements {
    /// Required skills
    pub skills:           Vec<String>,
    /// Experience level required
    pub experience_level: ExperienceLevel,
    /// Estimated delivery time in days
    pub delivery_days:    u32,
    /// Communication requirements
    pub communication:    Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExperienceLevel {
    Beginner,
    Intermediate,
    Expert,
}

/// Purchase transaction
#[derive(Debug, Clone)]
pub struct PurchaseTransaction {
    /// Transaction ID
    pub id:             String,
    /// Listing being purchased
    pub listing_id:     ListingId,
    /// Buyer peer ID
    pub buyer_peer_id:  String,
    /// Seller peer ID
    pub seller_peer_id: String,
    /// Purchase amount in satoshis
    pub amount_sats:    u64,
    /// Transaction timestamp
    pub timestamp:      u64,
    /// Transaction status
    pub status:         TransactionStatus,
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Transaction initiated
    Pending,
    /// Payment confirmed
    Paid,
    /// Content delivered
    Delivered,
    /// Transaction completed
    Completed,
    /// Transaction failed
    Failed,
    /// Transaction disputed
    Disputed,
}

/// Seller reputation score
#[derive(Debug, Clone)]
pub struct SellerReputation {
    /// Peer ID of seller
    pub peer_id:          String,
    /// Overall rating (0.0 to 5.0)
    pub rating:           f64,
    /// Number of reviews
    pub review_count:     u32,
    /// Success rate percentage
    pub success_rate:     f64,
    /// Total sales volume in satoshis
    pub total_sales_sats: u64,
}

/// Marketplace search query
#[derive(Debug, Clone)]
pub struct MarketplaceQuery {
    /// Search keywords
    pub keywords:    Option<String>,
    /// Category filter
    pub category:    Option<ListingCategory>,
    /// Price range in satoshis
    pub price_range: Option<(u64, u64)>,
    /// Minimum rating
    pub min_rating:  Option<f64>,
    /// Sort order
    pub sort_by:     SortOrder,
}

/// Sort order for search results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Sort by relevance
    Relevance,
    /// Sort by price (low to high)
    PriceAsc,
    /// Sort by price (high to low)
    PriceDesc,
    /// Sort by rating
    Rating,
    /// Sort by newest first
    Newest,
}

/// Search filters for marketplace queries
#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    /// Category filter
    pub category:              Option<ListingCategory>,
    /// Price range (min, max) in satoshis
    pub price_range:           Option<(u64, u64)>,
    /// Minimum rating (0.0 to 5.0)
    pub min_rating:            Option<f64>,
    /// Seller reputation threshold
    pub min_seller_reputation: Option<u32>,
    /// Listing status filter
    pub status:                Option<ListingStatus>,
}

/// Pagination parameters
#[derive(Debug, Clone)]
pub struct Pagination {
    /// Offset for results
    pub offset: usize,
    /// Maximum number of results
    pub limit:  usize,
}

impl Default for Pagination {
    fn default() -> Self {
        Self { offset: 0, limit: 20 }
    }
}

/// Search results
#[derive(Debug, Clone)]
pub struct SearchResults {
    /// Matching listings
    pub listings:    Vec<MarketplaceListing>,
    /// Total number of matches
    pub total_count: usize,
    /// Current page number
    pub page:        usize,
    /// Whether there are more results
    pub has_more:    bool,
}

/// Marketplace result type
pub type MarketplaceResult<T> = Result<T, crate::errors::MarketplaceError>;

// Re-exports
pub use delivery::*;
pub use orders::*;
pub use reviews::*;
pub use service::*;
