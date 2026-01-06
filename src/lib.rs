//! # Essentia Commerce Plugin
//!
//! Implements the decentralized e-commerce and affiliate platform logic,
//! including the Genesis Business Directory Node.

pub mod errors;
pub mod r#impl;
pub mod traits;
pub mod types;

// Re-exports for public API
pub use r#impl::*;
pub use types::*;
