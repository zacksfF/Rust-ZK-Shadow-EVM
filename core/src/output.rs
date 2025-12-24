//! Execution output types for Shadow-EVM
//!
//! Represents the result of EVM execution for ZK proof commitment.
//! The hash of ExecutionOutput is bound to the input for verification.

use crate::hashing::{compute_commitment, hash_struct};
use crate::state::InMemoryDB;
use crate::types::{Address, Gas, Hash};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

extern crate alloc;

/// EVM execution log entry
///
/// Represents an event emitted during contract execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Log {
    /// Contract that emitted the log
    pub address: Address,
    /// Log topics (up to 4, first is usually event signature)
    pub topics: Vec<Hash>,
    /// Log data (ABI-encoded event parameters)
    pub data: Vec<u8>,
}

impl Log {
    /// Create a new log entry
    pub fn new(address: Address, topics: Vec<Hash>, data: Vec<u8>) -> Self {
        Self {
            address,
            topics,
            data,
        }
    }

    /// Get the event signature (first topic, if present)
    pub fn event_signature(&self) -> Option<Hash> {
        self.topics.first().copied()
    }
}

/// Execution result status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Execution completed successfully
    Success,
    /// Execution reverted (REVERT opcode)
    Revert,
    /// Execution halted (out of gas, invalid opcode, etc.)
    Halt,
}

impl ExecutionStatus {
    /// Check if execution was successful
    pub fn is_success(&self) -> bool {
        matches!(self, ExecutionStatus::Success)
    }

    /// Check if execution reverted
    pub fn is_revert(&self) -> bool {
        matches!(self, ExecutionStatus::Revert)
    }

    /// Check if execution halted
    pub fn is_halt(&self) -> bool {
        matches!(self, ExecutionStatus::Halt)
    }
}

/// Complete execution output for Shadow-EVM
///
/// Contains all results from EVM execution that need to be committed to the ZK proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOutput {
    /// Execution status
    pub status: ExecutionStatus,
    /// Return data from execution
    pub return_data: Vec<u8>,
    /// Gas used during execution
    pub gas_used: Gas,
    /// Gas refunded (from storage clears, etc.)
    pub gas_refunded: Gas,
    /// Logs emitted during execution
    pub logs: Vec<Log>,
    /// Post-execution state
    pub post_state: InMemoryDB,
    /// Created contract address (if contract creation)
    pub created_address: Option<Address>,
}

impl ExecutionOutput {
    /// Create a successful execution output
    pub fn success(
        return_data: Vec<u8>,
        gas_used: Gas,
        gas_refunded: Gas,
        logs: Vec<Log>,
        post_state: InMemoryDB,
    ) -> Self {
        Self {
            status: ExecutionStatus::Success,
            return_data,
            gas_used,
            gas_refunded,
            logs,
            post_state,
            created_address: None,
        }
    }

    /// Create a reverted execution output
    pub fn revert(return_data: Vec<u8>, gas_used: Gas, post_state: InMemoryDB) -> Self {
        Self {
            status: ExecutionStatus::Revert,
            return_data,
            gas_used,
            gas_refunded: 0,
            logs: Vec::new(),
            post_state,
            created_address: None,
        }
    }

    /// Create a halted execution output
    pub fn halt(gas_used: Gas, post_state: InMemoryDB) -> Self {
        Self {
            status: ExecutionStatus::Halt,
            return_data: Vec::new(),
            gas_used,
            gas_refunded: 0,
            logs: Vec::new(),
            post_state,
            created_address: None,
        }
    }

    /// Compute deterministic hash of this output
    pub fn hash(&self) -> Hash {
        hash_struct(self)
    }

    /// Get the post-state root
    pub fn post_state_root(&self) -> Hash {
        self.post_state.compute_state_root()
    }

    /// Check if execution was successful
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    /// Check if execution reverted
    pub fn is_revert(&self) -> bool {
        self.status.is_revert()
    }

    /// Get effective gas used (accounting for refunds)
    pub fn effective_gas_used(&self) -> Gas {
        // Refund is capped at half of gas used
        let max_refund = self.gas_used / 2;
        let actual_refund = core::cmp::min(self.gas_refunded, max_refund);
        self.gas_used - actual_refund
    }

    /// Set the created contract address
    pub fn with_created_address(mut self, address: Address) -> Self {
        self.created_address = Some(address);
        self
    }
}

/// ZK Proof public commitment
///
/// This is what gets verified on-chain.
/// It binds the input to the output cryptographically.
///
/// # Structure
/// - `input_hash`: Keccak256 of ExecutionInput
/// - `output_hash`: Keccak256 of ExecutionOutput
/// - `commitment`: Keccak256(input_hash || output_hash)
///
/// The verifier only needs to check the commitment against the proof.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionCommitment {
    /// Hash of ExecutionInput
    pub input_hash: Hash,
    /// Hash of ExecutionOutput
    pub output_hash: Hash,
    /// Pre-state root
    pub pre_state_root: Hash,
    /// Post-state root
    pub post_state_root: Hash,
    /// Combined commitment (public input for ZK verifier)
    pub commitment: Hash,
}

impl ExecutionCommitment {
    /// Create commitment from input and output hashes
    pub fn new(
        input_hash: Hash,
        output_hash: Hash,
        pre_state_root: Hash,
        post_state_root: Hash,
    ) -> Self {
        let commitment = compute_commitment(&input_hash, &output_hash);
        Self {
            input_hash,
            output_hash,
            pre_state_root,
            post_state_root,
            commitment,
        }
    }

    /// Create commitment from execution input and output
    pub fn from_execution(input: &crate::input::ExecutionInput, output: &ExecutionOutput) -> Self {
        Self::new(
            input.hash(),
            output.hash(),
            input.pre_state_root(),
            output.post_state_root(),
        )
    }

    /// Verify that this commitment matches given input/output
    pub fn verify(&self, input_hash: &Hash, output_hash: &Hash) -> bool {
        self.input_hash == *input_hash
            && self.output_hash == *output_hash
            && self.commitment == compute_commitment(input_hash, output_hash)
    }

    /// Get the bytes for on-chain verification
    ///
    /// Returns the commitment in a format suitable for Solidity verification.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.commitment.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_creation() {
        let addr = Address::repeat_byte(0x01);
        let topic = Hash::repeat_byte(0x02);
        let data = vec![0x12, 0x34];

        let log = Log::new(addr, vec![topic], data.clone());

        assert_eq!(log.address, addr);
        assert_eq!(log.event_signature(), Some(topic));
        assert_eq!(log.data, data);
    }

    #[test]
    fn test_execution_status() {
        assert!(ExecutionStatus::Success.is_success());
        assert!(ExecutionStatus::Revert.is_revert());
        assert!(ExecutionStatus::Halt.is_halt());

        assert!(!ExecutionStatus::Success.is_revert());
        assert!(!ExecutionStatus::Revert.is_success());
    }

    #[test]
    fn test_execution_output_success() {
        let output =
            ExecutionOutput::success(vec![0x01, 0x02], 100_000, 5_000, vec![], InMemoryDB::new());

        assert!(output.is_success());
        assert!(!output.is_revert());
        assert_eq!(output.return_data, vec![0x01, 0x02]);
    }

    #[test]
    fn test_effective_gas_used() {
        let mut output = ExecutionOutput::success(
            vec![],
            100_000,
            60_000, // More than 50% - should be capped
            vec![],
            InMemoryDB::new(),
        );

        // Refund capped at 50% = 50,000
        assert_eq!(output.effective_gas_used(), 50_000);

        // Less than 50%
        output.gas_refunded = 20_000;
        assert_eq!(output.effective_gas_used(), 80_000);
    }

    #[test]
    fn test_commitment_deterministic() {
        let input_hash = Hash::repeat_byte(0x01);
        let output_hash = Hash::repeat_byte(0x02);
        let pre_root = Hash::repeat_byte(0x03);
        let post_root = Hash::repeat_byte(0x04);

        let c1 = ExecutionCommitment::new(input_hash, output_hash, pre_root, post_root);
        let c2 = ExecutionCommitment::new(input_hash, output_hash, pre_root, post_root);

        assert_eq!(c1, c2);
        assert_eq!(c1.commitment, c2.commitment);
    }

    #[test]
    fn test_commitment_verify() {
        let input_hash = Hash::repeat_byte(0x01);
        let output_hash = Hash::repeat_byte(0x02);

        let commitment = ExecutionCommitment::new(input_hash, output_hash, Hash::ZERO, Hash::ZERO);

        assert!(commitment.verify(&input_hash, &output_hash));
        assert!(!commitment.verify(&output_hash, &input_hash)); // Wrong order
    }

    #[test]
    fn test_commitment_different_inputs() {
        let input1 = Hash::repeat_byte(0x01);
        let input2 = Hash::repeat_byte(0x02);
        let output = Hash::repeat_byte(0x03);

        let c1 = ExecutionCommitment::new(input1, output, Hash::ZERO, Hash::ZERO);
        let c2 = ExecutionCommitment::new(input2, output, Hash::ZERO, Hash::ZERO);

        assert_ne!(c1.commitment, c2.commitment);
    }
}
