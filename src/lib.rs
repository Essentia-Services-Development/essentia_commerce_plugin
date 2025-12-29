//! # Essentia Commerce Plugin
//!
//! Implements the decentralized e-commerce and affiliate platform logic,
//! including the Genesis Business Directory Node.

pub mod types;
pub mod traits;
pub mod errors;
pub mod implementation;

// Re-exports for public API
pub use types::*;
pub use implementation::*;
