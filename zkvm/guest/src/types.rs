//! Type re-exports for ZKVM guest
//!
//! Re-exports all types from shadow-evm-core for convenient use in the guest.

#![allow(unused_imports)]

// Re-export all core types
pub use shadow_evm_core::prelude::*;

// Re-export additional types that might be needed
pub use shadow_evm_core::{compute_commitment, hash_struct, keccak256, VERSION};
