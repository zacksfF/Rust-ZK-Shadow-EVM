//! Host I/O helpers for RISC Zero guest
//!
//! Provides functions for reading inputs from the host and
//! committing outputs to the journal.

use risc0_zkvm::guest::env;
use shadow_evm_core::prelude::*;

/// Read ExecutionInput from the host
///
/// Deserializes the execution input that was provided by the host.
/// This input contains:
/// - Block environment (number, timestamp, gas limit, etc.)
/// - Transaction input (caller, to, value, data, etc.)
/// - Pre-execution state
pub fn read_input() -> ExecutionInput {
    env::read()
}

/// Commit the execution commitment to the journal
///
/// The journal is the public output of the ZK proof.
/// Anyone can read the journal after verification.
///
/// We commit the ExecutionCommitment which contains:
/// - Input hash
/// - Output hash  
/// - Pre-state root
/// - Post-state root
/// - Combined commitment
pub fn commit_output(commitment: &ExecutionCommitment) {
    env::commit(commitment);
}
