//! # Marketplace Order Types
//!
//! Types and structures for marketplace orders and transactions.

/// Unique order identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderId(String);

impl OrderId {
    pub fn new() -> Self {
        Self(format!("order-{}", essentia_uuid::Uuid::new_v4()))
    }
}

/// A marketplace order
#[derive(Debug, Clone)]
pub struct Order {
    /// Unique order ID
    pub id:           OrderId,
    /// Listing purchased
    pub listing_id:   super::ListingId,
    /// Buyer node
    pub buyer:        String, // Placeholder for PeerNodeId
    /// Seller node
    pub seller:       String, // Placeholder for PeerNodeId
    /// Order status
    pub status:       OrderStatus,
    /// Total price paid (sats)
    pub total_sats:   u64,
    /// Payment hash (Lightning)
    pub payment_hash: Option<String>,
    /// Escrow status (for services)
    pub escrow:       Option<OrderEscrow>,
    /// Created timestamp
    pub created_at:   u64,
    /// Paid timestamp
    pub paid_at:      Option<u64>,
    /// Delivered timestamp
    pub delivered_at: Option<u64>,
    /// Completed timestamp
    pub completed_at: Option<u64>,
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    /// Order created, awaiting payment
    Pending,
    /// Payment received, processing
    Paid,
    /// Digital content delivered
    Delivered,
    /// Service work in progress
    InProgress,
    /// Service completed, awaiting approval
    Completed,
    /// Order cancelled
    Cancelled,
    /// Dispute raised
    Disputed,
    /// Dispute resolved
    Resolved,
}

/// Escrow information for service orders
#[derive(Debug, Clone)]
pub struct OrderEscrow {
    /// Escrow amount held (sats)
    pub amount_sats:        u64,
    /// Release conditions
    pub release_conditions: Vec<ReleaseCondition>,
    /// Current escrow status
    pub status:             EscrowStatus,
}

/// Escrow release conditions
#[derive(Debug, Clone)]
pub enum ReleaseCondition {
    /// Buyer approval required
    BuyerApproval,
    /// Milestone completion
    Milestone { milestone_id: String },
    /// Time-based auto-release
    TimeBased { release_at: u64 },
    /// Third-party arbitration
    Arbitration { arbitrator: String },
}

/// Escrow status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscrowStatus {
    /// Funds held in escrow
    Held,
    /// Partially released
    PartialRelease,
    /// Fully released to seller
    Released,
    /// Returned to buyer
    Refunded,
    /// In dispute resolution
    Disputed,
}

/// Order review/rating
#[derive(Debug, Clone)]
pub struct OrderReview {
    /// Order being reviewed
    pub order_id:      OrderId,
    /// Reviewer (buyer)
    pub reviewer:      String,
    /// Rating (1-5 stars)
    pub rating:        u8,
    /// Review text
    pub review_text:   String,
    /// Created timestamp
    pub created_at:    u64,
    /// Helpful votes
    pub helpful_votes: u64,
}

/// Dispute information
#[derive(Debug, Clone)]
pub struct OrderDispute {
    /// Order in dispute
    pub order_id:    OrderId,
    /// Dispute raiser
    pub raised_by:   String,
    /// Dispute reason
    pub reason:      DisputeReason,
    /// Dispute description
    pub description: String,
    /// Evidence attachments
    pub evidence:    Vec<DisputeEvidence>,
    /// Resolution
    pub resolution:  Option<DisputeResolution>,
    /// Created timestamp
    pub created_at:  u64,
}

/// Dispute reason
#[derive(Debug, Clone)]
pub enum DisputeReason {
    /// Seller didn't deliver
    NonDelivery,
    /// Product/service quality issues
    QualityIssue,
    /// Late delivery
    LateDelivery,
    /// Communication problems
    CommunicationIssue,
    /// Payment not received
    PaymentIssue,
    /// Other
    Other { description: String },
}

/// Dispute evidence
#[derive(Debug, Clone)]
pub struct DisputeEvidence {
    /// Evidence type
    pub evidence_type: EvidenceType,
    /// Description
    pub description:   String,
    /// File hash or URL
    pub reference:     String,
}

/// Evidence type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceType {
    Screenshot,
    MessageLog,
    File,
    Video,
    Other,
}

/// Dispute resolution
#[derive(Debug, Clone)]
pub struct DisputeResolution {
    /// Resolution decision
    pub decision:      DisputeDecision,
    /// Resolution amount (sats to buyer/seller)
    pub buyer_amount:  u64,
    pub seller_amount: u64,
    /// Resolution notes
    pub notes:         String,
    /// Resolved timestamp
    pub resolved_at:   u64,
}

/// Dispute decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisputeDecision {
    /// Refund buyer fully
    RefundBuyer,
    /// Release to seller fully
    ReleaseToSeller,
    /// Partial refund
    PartialRefund,
    /// Escalate to arbitration
    Arbitration,
}
