//! Shadow-EVM ZKVM Guest Entry Point
//!
//! This is the main program that runs inside the RISC-V ZK-VM.
//! It reads an ExecutionInput from the host, executes the EVM transaction,
//! and commits the ExecutionCommitment to the journal.
//!
//! # Flow
//! 1. Host serializes ExecutionInput and passes it to the guest
//! 2. Guest deserializes and executes using ShadowExecutor
//! 3. Guest computes ExecutionCommitment
//! 4. Guest commits the commitment to the journal
//! 5. Prover generates a proof of correct execution
//!
//! # Public Output (Journal)
//! The journal contains the ExecutionCommitment which includes:
//! - input_hash: Hash of the execution input
//! - output_hash: Hash of the execution output
//! - pre_state_root: Merkle root of pre-execution state
//! - post_state_root: Merkle root of post-execution state
//! - commitment: Combined cryptographic commitment

#![no_main]
#![no_std]

extern crate alloc;

mod evm;
mod types;

use evm::{commit_output, read_input, ZkExecutor};

risc0_zkvm::guest::entry!(main);

/// Guest main entry point
///
/// This function is called when the ZK-VM starts executing.
/// Any panic will cause the proof generation to fail.
fn main() {
    // Step 1: Read the execution input from the host
    let input = read_input();

    // Step 2: Execute the EVM transaction
    // This uses the core ShadowExecutor which wraps revm
    let result = ZkExecutor::execute(input);

    // Step 3: Handle the result and commit to journal
    match result {
        Ok((_output, commitment)) => {
            // Commit the execution commitment to the journal
            // This becomes the public output of the ZK proof
            commit_output(&commitment);
        }
        Err(_err) => {
            // Execution failed - panic to abort proof generation
            // The host should validate inputs before proving
            panic!("EVM execution failed");
        }
    }
}
