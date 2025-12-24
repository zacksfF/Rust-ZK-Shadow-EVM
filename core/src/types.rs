//! Core type definitions for Shadow-EVM
//!
//! Uses alloy-primitives for Ethereum-compatible types.
//! These types are designed to be ZK-VM friendly (no_std compatible).

pub use alloy_primitives::{Address, B256, Bytes, U256};

/// 32-byte hash (Keccak256 output)
pub type Hash = B256;

/// Account nonce type
pub type Nonce = u64;

/// Gas amount type
pub type Gas = u64;

/// Block number
pub type BlockNumber = u64;

/// Timestamp (seconds since epoch)
pub type Timestamp = u64;

/// Wei amount (for clarity in value transfers)
pub type Wei = U256;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_creation() {
        let addr = Address::repeat_byte(0x42);
        assert_eq!(addr.as_slice()[0], 0x42);
    }

    #[test]
    fn test_hash_zero() {
        let hash = Hash::ZERO;
        assert_eq!(hash.as_slice(), &[0u8; 32]);
    }

    #[test]
    fn test_u256_arithmetic() {
        let a = U256::from(100u64);
        let b = U256::from(50u64);
        assert_eq!(a - b, U256::from(50u64));
    }
}
