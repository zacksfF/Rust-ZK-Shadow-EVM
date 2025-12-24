//! # Shadow-EVM Core
//!
//! Zero-Knowledge Coprocessor for verifiable off-chain Ethereum execution.
//!
//! This crate provides the core EVM execution logic that can run inside a ZK-VM.
//! It is designed to be:
//! - **Deterministic**: Same input always produces same output
//! - **ZK-friendly**: Compatible with `no_std` for RISC-V ZK-VMs
//! - **Auditable**: Clean separation of concerns
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────┐
//! │ ExecutionInput  │ ── Block, Tx, Pre-State
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │ ShadowExecutor  │ ── Wraps revm
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │ ExecutionOutput │ ── Return data, logs, post-state
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │ExecutionCommit- │ ── Keccak256(input || output)
//! │     ment        │    for ZK proof binding
//! └─────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use shadow_evm_core::{
//!     ExecutionInput, ExecutionBuilder, ShadowExecutor,
//!     BlockEnv, TxInput, InMemoryDB, AccountState,
//! };
//!
//! // Setup pre-state
//! let mut state = InMemoryDB::new();
//! state.insert_account(sender, AccountState::new_with_balance(balance));
//!
//! // Execute transaction
//! let (output, commitment) = ExecutionBuilder::new()
//!     .with_state(state)
//!     .with_tx(TxInput::transfer(sender, receiver, amount))
//!     .execute()?;
//!
//! // Use commitment for ZK proof
//! println!("Commitment: {:?}", commitment.commitment);
//! ```
//!
//! ## Modules
//!
//! - [`types`] - Core type definitions (Address, Hash, U256)
//! - [`errors`] - Error types and Result alias
//! - [`hashing`] - Keccak256 and commitment functions
//! - [`state`] - In-memory state database
//! - [`input`] - Execution input types
//! - [`output`] - Execution output and commitment types
//! - [`evm`] - Core EVM executor

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod errors;
pub mod evm;
pub mod hashing;
pub mod input;
pub mod output;
pub mod state;
pub mod types;

// Re-exports for convenience
pub use errors::{Result, ShadowEvmError};
pub use evm::{ExecutionBuilder, ShadowExecutor};
pub use hashing::{compute_commitment, hash_struct, keccak256};
pub use input::{BlockEnv, ExecutionInput, TxInput};
pub use output::{ExecutionCommitment, ExecutionOutput, ExecutionStatus, Log};
pub use state::{AccountState, InMemoryDB};
pub use types::{Address, Bytes, Gas, Hash, U256};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        AccountState, Address, BlockEnv, Bytes, ExecutionBuilder, ExecutionCommitment,
        ExecutionInput, ExecutionOutput, ExecutionStatus, Gas, Hash, InMemoryDB, Log, Result,
        ShadowEvmError, ShadowExecutor, TxInput, U256,
    };
}

#[cfg(test)]
mod integration_tests {
    use super::prelude::*;

    /// End-to-end test: deploy contract
    #[test]
    fn test_deploy_and_call() {
        let deployer = Address::repeat_byte(0x01);

        // Simple init code that returns 0x42 as the contract code
        // PUSH1 0x42, PUSH1 0, MSTORE8, PUSH1 1, PUSH1 0, RETURN
        let init_code = vec![
            0x60, 0x42, // PUSH1 0x42 (the byte we want as code)
            0x60, 0x00, // PUSH1 0
            0x53, // MSTORE8 (store single byte at memory[0])
            0x60, 0x01, // PUSH1 1 (return 1 byte)
            0x60, 0x00, // PUSH1 0 (from offset 0)
            0xf3, // RETURN
        ];

        // Setup state
        let mut state = InMemoryDB::new();
        state.insert_account(
            deployer,
            AccountState::new_with_balance(U256::from(10u128.pow(18))),
        );

        // Deploy contract
        let deploy_input = ExecutionInput::new(
            BlockEnv::default(),
            TxInput::create(deployer, init_code, U256::ZERO)
                .with_gas_limit(1_000_000)
                .with_nonce(0),
            state,
        );

        let (deploy_output, commitment) = ShadowExecutor::execute(deploy_input).unwrap();
        assert!(deploy_output.is_success(), "Deploy should succeed");

        let contract_addr = deploy_output
            .created_address
            .expect("Should have created address");

        // Verify contract exists in post_state
        assert!(deploy_output
            .post_state
            .accounts
            .contains_key(&contract_addr));

        // Verify contract has code stored
        let contract = deploy_output
            .post_state
            .accounts
            .get(&contract_addr)
            .unwrap();
        assert!(!contract.code.is_empty(), "Contract should have code");

        // First byte should be 0x42
        assert_eq!(
            contract.code[0], 0x42,
            "Contract code should start with 0x42"
        );

        // Verify commitment is valid
        assert!(!commitment.commitment.is_zero());
        assert_ne!(commitment.pre_state_root, commitment.post_state_root);
    }

    /// Test that pre-state and post-state roots differ after modification
    #[test]
    fn test_state_root_changes() {
        let sender = Address::repeat_byte(0x01);
        let receiver = Address::repeat_byte(0x02);

        let mut state = InMemoryDB::new();
        state.insert_account(
            sender,
            AccountState::new_with_balance(U256::from(10u128.pow(18))),
        );

        let pre_root = state.compute_state_root();

        let input = ExecutionInput::new(
            BlockEnv::default(),
            TxInput::transfer(sender, receiver, U256::from(1000u64)),
            state,
        );

        let (output, commitment) = ShadowExecutor::execute(input).unwrap();

        let post_root = output.post_state.compute_state_root();

        // Roots should differ
        assert_ne!(pre_root, post_root);
        assert_eq!(commitment.pre_state_root, pre_root);
        assert_eq!(commitment.post_state_root, post_root);
    }
}
