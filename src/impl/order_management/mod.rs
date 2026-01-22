//! Order management system.
//!
//! This module provides a complete e-commerce order management system
//! with payment processing, fulfillment tracking, and blockchain settlement.
//!
//! The module is organized according to EMD (Entity-Module-Data) pattern:
//! - `types/`: All type definitions and implementations
//! - `implementations/`: Business logic implementations
//! - `errors/`: Error types and handling

pub mod types {
    //! Type definitions for order management.

    pub mod basic_types;
    pub mod order_types;
    pub mod main_order_types;
    pub mod service_types;

    // Re-export commonly used types
    pub use basic_types::*;
    pub use order_types::*;
    pub use main_order_types::*;
    pub use service_types::*;
}

pub mod implementations {
    //! Business logic implementations.

    pub mod order_impl;
    pub mod service_impl;

    // Re-export implementations
    // pub use order_impl::*;
    // pub use service_impl::*;
}

pub mod errors {
    //! Error types and handling.

    pub mod r#mod;

    // Re-export errors
    pub use r#mod::*;
}

// Re-export main types for convenience
pub use types::*;
pub use implementations::*;
