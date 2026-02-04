//! # Inventory Sync Implementation (GAP-220-D-004)
//!
//! Implementation of real-time inventory synchronization and management.

pub use crate::types::inventory_sync::*;

mod service;

#[cfg(test)]
mod tests;
