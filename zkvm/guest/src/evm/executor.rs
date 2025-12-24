//! ZK Executor for guest execution
//!
//! Wraps the core ShadowExecutor for use inside the ZK-VM guest.

use shadow_evm_core::prelude::*;

/// ZK Executor wrapper
///
/// Provides a streamlined interface for executing EVM transactions
/// inside the ZK-VM and generating commitments.
pub struct ZkExecutor;

impl ZkExecutor {
    /// Execute a transaction and return the commitment
    ///
    /// This is the main entry point for ZK execution.
    /// It takes an ExecutionInput and returns either:
    /// - Ok((output, commitment)) on success
    /// - Err(error) on failure
    ///
    /// # ZK Properties
    /// - Execution is deterministic
    /// - Same input always produces same output
    /// - Commitment binds input to output cryptographically
    pub fn execute(input: ExecutionInput) -> Result<(ExecutionOutput, ExecutionCommitment)> {
        // Use the core executor
        ShadowExecutor::execute(input)
    }
}
