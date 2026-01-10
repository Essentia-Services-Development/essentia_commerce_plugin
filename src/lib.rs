//! # Essentia Commerce Plugin
//!
//! Implements the decentralized e-commerce and affiliate platform logic,
//! including the Genesis Business Directory Node.

#![allow(clippy::unnecessary_literal_bound)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::struct_excessive_bools)]

pub mod errors;
pub mod r#impl;
pub mod traits;
pub mod types;

// Re-exports for public API
pub use r#impl::*;
pub use types::*;
