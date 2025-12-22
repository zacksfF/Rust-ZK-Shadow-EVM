/* 
Error types for Shadow-EVM execution
Provides comprehensive error handling for all execution scenarios.
*/

use crate::types::{Address, Hash};
use thiserror::Error;

/// Errors that can occur during Shadow-EVM execution
#[derive(Debug, Error)]
pub enum ShadowEvmError {
    /// Account not found in state
    #[error("account not found: {0}")]
    AccountNotFound(Address),

    /// Storage slot not found
    #[error("storage not found for {account} at slot {slot}")]
    StorageNotFound { account: Address, slot: Hash },

    /// EVM execution reverted
    #[error("execution reverted: {0}")]
    ExecutionReverted(String),

    /// EVM execution halted (out of gas, invalid opcode, etc.)
    #[error("execution halted: {0}")]
    ExecutionHalted(String),

    /// Invalid bytecode
    #[error("invalid bytecode: {0}")]
    InvalidBytecode(String),

    /// Serialization error
    #[error("serialization error: {0}")]
    SerializationError(String),

    /// State commitment mismatch
    #[error("state commitment mismatch: expected {expected}, got {actual}")]
    CommitmentMismatch { expected: Hash, actual: Hash },

    /// Invalid transaction parameters
    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),

    /// Database error
    #[error("database error: {0}")]
    DatabaseError(String),
}

/// Result type for Shadow-EVM operations
pub type Result<T> = core::result::Result<T, ShadowEvmError>;

impl From<bincode::Error> for ShadowEvmError {
    fn from(err: bincode::Error) -> Self {
        ShadowEvmError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ShadowEvmError::AccountNotFound(Address::ZERO);
        assert!(err.to_string().contains("account not found"));
    }

    #[test]
    fn test_commitment_mismatch_error() {
        let err = ShadowEvmError::CommitmentMismatch {
            expected: Hash::ZERO,
            actual: Hash::repeat_byte(0x01),
        };
        assert!(err.to_string().contains("commitment mismatch"));
    }
}
