//! # Essentia Commerce Plugin
//!
//! Implements the decentralized e-commerce and affiliate platform logic,
//! including the Genesis Business Directory Node.

#![allow(clippy::unnecessary_literal_bound)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(missing_docs)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::unnecessary_sort_by)]
#![allow(clippy::missing_panics_doc)]

pub mod errors;
pub mod implementation;
pub mod marketplace;
pub mod traits;
pub mod types;

// Re-exports for public API
pub use implementation::CommerceFlexForgeIntegration;
pub use types::{CommerceConfig, GenesisDirectory};
