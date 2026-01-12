//! Content delivery service for marketplace purchases

use std::collections::HashMap;

use crate::errors::MarketplaceError;

/// Content delivery service result type
pub type DeliveryResult<T> = Result<T, MarketplaceError>;

/// Unique content hash identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentHash(String);

impl ContentHash {
    /// Create from string hash
    pub fn new(hash: String) -> Self {
        Self(hash)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Access token for content delivery
#[derive(Debug, Clone)]
pub struct AccessToken {
    /// Token string
    pub token:          String,
    /// Buyer peer ID
    pub buyer:          String, // Placeholder for PeerNodeId
    /// Listing ID
    pub listing_id:     super::ListingId,
    /// Content hash
    pub content_hash:   ContentHash,
    /// Granted timestamp
    pub granted_at:     u64,
    /// Expiration timestamp
    pub expires_at:     Option<u64>,
    /// Download count
    pub download_count: u32,
    /// Maximum downloads allowed
    pub max_downloads:  Option<u32>,
}

/// Download information
#[derive(Debug, Clone)]
pub struct DownloadInfo {
    /// Content hash
    pub content_hash: ContentHash,
    /// Available providers
    pub providers:    Vec<String>, // Placeholder for PeerNodeId
    /// Access token
    pub token:        String,
}

/// Content delivery service
pub struct ContentDeliveryService {
    /// Access tokens by (buyer, listing_id)
    access_tokens: HashMap<(String, super::ListingId), AccessToken>,
    /// Content providers by content hash
    providers:     HashMap<ContentHash, Vec<String>>,
}

impl ContentDeliveryService {
    /// Create new content delivery service
    pub fn new() -> Self {
        Self { access_tokens: HashMap::new(), providers: HashMap::new() }
    }

    /// Register content provider
    pub fn register_provider(&mut self, content_hash: ContentHash, provider: String) {
        self.providers.entry(content_hash).or_default().push(provider);
    }

    /// Grant access after purchase
    pub fn grant_access(
        &mut self, buyer: String, listing_id: super::ListingId, content_hash: ContentHash,
    ) -> DeliveryResult<AccessToken> {
        let token = AccessToken {
            token:          generate_secure_token(),
            buyer:          buyer.clone(),
            listing_id:     listing_id.clone(),
            content_hash:   content_hash.clone(),
            granted_at:     current_timestamp(),
            expires_at:     None, // No expiration for now
            download_count: 0,
            max_downloads:  Some(5), // Allow 5 downloads
        };

        self.access_tokens.insert((buyer, listing_id), token.clone());

        Ok(token)
    }

    /// Verify access and get download URL
    pub fn get_download(&mut self, token: &str, buyer: &str) -> DeliveryResult<DownloadInfo> {
        // Find token
        let access = self
            .access_tokens
            .values_mut()
            .find(|t| t.token == token && t.buyer == buyer)
            .ok_or(MarketplaceError::InvalidToken)?;

        // Check expiry
        if let Some(expires) = access.expires_at {
            if current_timestamp() > expires {
                return Err(MarketplaceError::TokenExpired);
            }
        }

        // Check download limit
        if let Some(max) = access.max_downloads {
            if access.download_count >= max {
                return Err(MarketplaceError::DownloadLimitReached);
            }
        }

        access.download_count += 1;

        // Find providers
        let providers =
            self.providers.get(&access.content_hash).ok_or(MarketplaceError::NoProviders)?;

        Ok(DownloadInfo {
            content_hash: access.content_hash.clone(),
            providers:    providers.clone(),
            token:        token.to_string(),
        })
    }

    /// Revoke access (for refunds/disputes)
    pub fn revoke_access(&mut self, buyer: &str, listing_id: &super::ListingId) {
        self.access_tokens.remove(&(buyer.to_string(), listing_id.clone()));
    }
}

impl Default for ContentDeliveryService {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate secure access token
fn generate_secure_token() -> String {
    format!("tok_{}", essentia_uuid::Uuid::new_v4())
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}
