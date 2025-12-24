//! Hashing utilities for Shadow-EVM
//!
//! Provides Keccak256 hashing for state commitments and input/output binding.
//! All hashing functions are deterministic for ZK proof compatibility.

use crate::types::{Hash, B256};
use sha3::{Digest, Keccak256};

/// Compute Keccak256 hash of arbitrary data
///
/// # Arguments
/// * `data` - Byte slice to hash
///
/// # Returns
/// 32-byte Keccak256 hash
pub fn keccak256(data: &[u8]) -> Hash {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    B256::from_slice(&hasher.finalize())
}

/// Compute commitment hash for binding input to output
///
/// This is the public input for ZK proof verification:
/// `commitment = keccak256(input_hash || output_hash)`
///
/// # Arguments
/// * `input_hash` - Hash of the ExecutionInput
/// * `output_hash` - Hash of the ExecutionOutput
///
/// # Returns
/// Combined commitment hash for on-chain verification
pub fn compute_commitment(input_hash: &Hash, output_hash: &Hash) -> Hash {
    let mut data = [0u8; 64];
    data[..32].copy_from_slice(input_hash.as_slice());
    data[32..].copy_from_slice(output_hash.as_slice());
    keccak256(&data)
}

/// Hash a serializable struct for commitment
///
/// Uses bincode for deterministic serialization before hashing.
/// This ensures consistent hashes across different executions.
///
/// # Arguments
/// * `value` - Any serializable value
///
/// # Returns
/// Keccak256 hash of the bincode-serialized value
pub fn hash_struct<T: serde::Serialize>(value: &T) -> Hash {
    let bytes = bincode::serialize(value).expect("serialization should not fail");
    keccak256(&bytes)
}

/// Hash multiple values together
///
/// Concatenates all byte slices and hashes the result.
pub fn hash_concat(values: &[&[u8]]) -> Hash {
    let mut hasher = Keccak256::new();
    for value in values {
        hasher.update(value);
    }
    B256::from_slice(&hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keccak256_empty() {
        let hash = keccak256(&[]);
        // Known empty Keccak256 hash
        assert_eq!(
            hex::encode(hash),
            "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
        );
    }

    #[test]
    fn test_keccak256_hello() {
        let hash = keccak256(b"hello");
        assert_eq!(
            hex::encode(hash),
            "1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8"
        );
    }

    #[test]
    fn test_commitment_deterministic() {
        let input = B256::ZERO;
        let output = B256::ZERO;
        let c1 = compute_commitment(&input, &output);
        let c2 = compute_commitment(&input, &output);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_commitment_different_inputs() {
        let input1 = B256::ZERO;
        let input2 = B256::repeat_byte(0x01);
        let output = B256::ZERO;

        let c1 = compute_commitment(&input1, &output);
        let c2 = compute_commitment(&input2, &output);
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_hash_struct() {
        #[derive(serde::Serialize)]
        struct TestStruct {
            a: u64,
            b: u64,
        }

        let s1 = TestStruct { a: 1, b: 2 };
        let s2 = TestStruct { a: 1, b: 2 };
        let s3 = TestStruct { a: 2, b: 1 };

        assert_eq!(hash_struct(&s1), hash_struct(&s2));
        assert_ne!(hash_struct(&s1), hash_struct(&s3));
    }

    #[test]
    fn test_hash_concat() {
        let a = b"hello";
        let b = b"world";

        let h1 = hash_concat(&[a, b]);
        let h2 = keccak256(b"helloworld");

        assert_eq!(h1, h2);
    }
}
