//! Core EVM executor for Shadow-EVM
//!
//! Wraps `revm` to execute EVM bytecode deterministically.
//! This is the heart of the Shadow-EVM coprocessor.

extern crate alloc;

use crate::errors::{Result, ShadowEvmError};
use crate::input::{BlockEnv, ExecutionInput, TxInput};
use crate::output::{ExecutionCommitment, ExecutionOutput, Log};
use crate::state::{AccountState, InMemoryDB};
use crate::types::U256;
use alloc::format;
use alloc::vec::Vec;
use revm::primitives::{BlockEnv as RevmBlockEnv, CfgEnv, SpecId, TxEnv, TxKind};
use revm::Evm;

/// Shadow-EVM Executor
///
/// Deterministic EVM execution engine for ZK proof generation.
/// Wraps `revm` with configuration suitable for off-chain proving.
///
/// # Design Principles
/// - **Determinism**: Same input always produces same output
/// - **Isolation**: No external state access
/// - **Compatibility**: Full EVM support via revm
pub struct ShadowExecutor;

impl ShadowExecutor {
    /// Execute EVM transaction and return output with commitment
    ///
    /// This is the main entry point for Shadow-EVM execution.
    ///
    /// # Arguments
    /// * `input` - Complete execution input including state, block, and transaction
    ///
    /// # Returns
    /// * `Ok((output, commitment))` - Execution output and ZK commitment
    /// * `Err(e)` - Execution error
    ///
    /// # Example
    /// ```rust,ignore
    /// let input = ExecutionInput::new(block, tx, pre_state);
    /// let (output, commitment) = ShadowExecutor::execute(input)?;
    /// ```
    pub fn execute(input: ExecutionInput) -> Result<(ExecutionOutput, ExecutionCommitment)> {
        let input_hash = input.hash();
        let pre_state_root = input.pre_state_root();

        // Build and configure revm
        let mut evm = Self::build_evm(input.block, input.tx.clone(), input.pre_state.clone())?;

        // Execute transaction using transact()
        let result = evm
            .transact()
            .map_err(|e| ShadowEvmError::ExecutionHalted(format!("{:?}", e)))?;

        // Apply state changes to our database
        let mut post_state = input.pre_state;
        Self::apply_state_changes(&mut post_state, &result)?;

        // Build output from result
        let output = Self::build_output(result.result, post_state)?;

        // Create cryptographic commitment
        let output_hash = output.hash();
        let post_state_root = output.post_state_root();
        let commitment =
            ExecutionCommitment::new(input_hash, output_hash, pre_state_root, post_state_root);

        Ok((output, commitment))
    }

    /// Execute without committing state changes
    ///
    /// Useful for dry-runs and gas estimation.
    pub fn simulate(input: ExecutionInput) -> Result<ExecutionOutput> {
        let mut evm = Self::build_evm(input.block, input.tx, input.pre_state.clone())?;

        let result = evm
            .transact()
            .map_err(|e| ShadowEvmError::ExecutionHalted(format!("{:?}", e)))?;

        // Use the original pre_state for simulation output
        Self::build_output(result.result, input.pre_state)
    }

    /// Build revm EVM instance with Shadow-EVM configuration
    fn build_evm(
        block: BlockEnv,
        tx: TxInput,
        db: InMemoryDB,
    ) -> Result<Evm<'static, (), InMemoryDB>> {
        // Configure chain - use Cancun (latest stable) spec
        let mut cfg = CfgEnv::default();
        cfg.chain_id = block.chain_id;

        // Configure block environment
        let block_env = RevmBlockEnv {
            number: U256::from(block.number),
            timestamp: U256::from(block.timestamp),
            gas_limit: U256::from(block.gas_limit),
            coinbase: block.coinbase,
            basefee: block.base_fee,
            prevrandao: Some(block.prev_randao),
            ..Default::default()
        };

        // Configure transaction environment
        let tx_env = TxEnv {
            caller: tx.caller,
            transact_to: match tx.to {
                Some(addr) => TxKind::Call(addr),
                None => TxKind::Create,
            },
            value: tx.value,
            data: tx.data.into(),
            gas_limit: tx.gas_limit,
            gas_price: tx.gas_price,
            nonce: Some(tx.nonce),
            ..Default::default()
        };

        // Build EVM with Cancun spec (for latest features)
        let evm = Evm::builder()
            .with_db(db)
            .with_spec_id(SpecId::CANCUN)
            .modify_cfg_env(|c| *c = cfg)
            .modify_block_env(|b| *b = block_env)
            .modify_tx_env(|t| *t = tx_env)
            .build();

        Ok(evm)
    }

    /// Apply state changes from execution result to our database
    fn apply_state_changes(
        db: &mut InMemoryDB,
        result: &revm::primitives::ResultAndState,
    ) -> Result<()> {
        // Apply balance changes and state changes
        for (addr, account) in &result.state {
            let info = &account.info;

            if let Some(existing) = db.get_account_mut(addr) {
                // Update existing account
                existing.balance = info.balance;
                existing.nonce = info.nonce;
                // Update storage
                for (slot, value) in &account.storage {
                    existing.set_storage(*slot, value.present_value);
                }
            } else {
                // Create new account
                let mut new_acc = AccountState::new_with_balance(info.balance);
                new_acc.nonce = info.nonce;
                if let Some(code) = &info.code {
                    new_acc.code = code.bytes().to_vec();
                    new_acc.code_hash = info.code_hash;
                }
                // Add storage
                for (slot, value) in &account.storage {
                    new_acc.set_storage(*slot, value.present_value);
                }
                db.insert_account(*addr, new_acc);
            }
        }

        Ok(())
    }

    /// Build execution output from revm result
    fn build_output(
        result: revm::primitives::ExecutionResult,
        db: InMemoryDB,
    ) -> Result<ExecutionOutput> {
        match result {
            revm::primitives::ExecutionResult::Success {
                output,
                gas_used,
                gas_refunded,
                logs,
                ..
            } => {
                let (return_data, created_address) = match output {
                    revm::primitives::Output::Call(data) => (data.to_vec(), None),
                    revm::primitives::Output::Create(data, addr) => (data.to_vec(), addr),
                };

                let logs = Self::convert_logs(logs);

                let mut output =
                    ExecutionOutput::success(return_data, gas_used, gas_refunded, logs, db);

                if let Some(addr) = created_address {
                    output = output.with_created_address(addr);
                }

                Ok(output)
            }
            revm::primitives::ExecutionResult::Revert { output, gas_used } => {
                Ok(ExecutionOutput::revert(output.to_vec(), gas_used, db))
            }
            revm::primitives::ExecutionResult::Halt { gas_used, .. } => {
                Ok(ExecutionOutput::halt(gas_used, db))
            }
        }
    }

    /// Convert revm logs to Shadow-EVM logs
    fn convert_logs(logs: Vec<revm::primitives::Log>) -> Vec<Log> {
        logs.into_iter()
            .map(|log| Log::new(log.address, log.topics().to_vec(), log.data.data.to_vec()))
            .collect()
    }
}

/// Builder for creating execution inputs
pub struct ExecutionBuilder {
    block: BlockEnv,
    tx: TxInput,
    state: InMemoryDB,
}

impl ExecutionBuilder {
    /// Create a new execution builder
    pub fn new() -> Self {
        Self {
            block: BlockEnv::default(),
            tx: TxInput::default(),
            state: InMemoryDB::new(),
        }
    }

    /// Set the block environment
    pub fn with_block(mut self, block: BlockEnv) -> Self {
        self.block = block;
        self
    }

    /// Set the transaction input
    pub fn with_tx(mut self, tx: TxInput) -> Self {
        self.tx = tx;
        self
    }

    /// Set the pre-state
    pub fn with_state(mut self, state: InMemoryDB) -> Self {
        self.state = state;
        self
    }

    /// Build the execution input
    pub fn build(self) -> ExecutionInput {
        ExecutionInput::new(self.block, self.tx, self.state)
    }

    /// Execute immediately
    pub fn execute(self) -> Result<(ExecutionOutput, ExecutionCommitment)> {
        ShadowExecutor::execute(self.build())
    }
}

impl Default for ExecutionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AccountState;
    use crate::types::{Address, U256};

    /// Test simple ETH transfer between accounts
    #[test]
    fn test_simple_transfer() {
        // Setup: sender with 1 ETH
        let sender = Address::repeat_byte(0x01);
        let receiver = Address::repeat_byte(0x02);
        let initial_balance = U256::from(10u128.pow(18)); // 1 ETH
        let transfer_amount = U256::from(10u128.pow(17)); // 0.1 ETH

        let mut pre_state = InMemoryDB::new();
        pre_state.insert_account(sender, AccountState::new_with_balance(initial_balance));

        // Execute transfer
        let input = ExecutionInput::new(
            BlockEnv::default(),
            TxInput::transfer(sender, receiver, transfer_amount),
            pre_state,
        );

        let (output, commitment) = ShadowExecutor::execute(input).unwrap();

        // Verify success
        assert!(output.is_success());
        assert_eq!(output.gas_used, 21000); // Standard transfer

        // Verify balances
        let receiver_balance = output
            .post_state
            .accounts
            .get(&receiver)
            .map(|a| a.balance)
            .unwrap_or(U256::ZERO);
        assert_eq!(receiver_balance, transfer_amount);

        // Verify commitment is valid
        assert!(!commitment.commitment.is_zero());
        assert_ne!(commitment.pre_state_root, commitment.post_state_root);
    }

    /// Test contract deployment
    #[test]
    fn test_contract_creation() {
        let deployer = Address::repeat_byte(0x01);

        // Simple contract: PUSH1 0x42, PUSH1 0, MSTORE, PUSH1 1, PUSH1 31, RETURN
        // Returns 0x42
        let init_code = vec![
            0x60, 0x42, // PUSH1 0x42
            0x60, 0x00, // PUSH1 0
            0x52, // MSTORE
            0x60, 0x01, // PUSH1 1
            0x60, 0x1f, // PUSH1 31
            0xf3, // RETURN
        ];

        let mut pre_state = InMemoryDB::new();
        pre_state.insert_account(
            deployer,
            AccountState::new_with_balance(U256::from(10u128.pow(18))),
        );

        let input = ExecutionInput::new(
            BlockEnv::default(),
            TxInput::create(deployer, init_code, U256::ZERO).with_gas_limit(100_000),
            pre_state,
        );

        let (output, _commitment) = ShadowExecutor::execute(input).unwrap();

        // Should succeed
        assert!(output.is_success());

        // Should have created address
        assert!(output.created_address.is_some());

        // Created contract should exist in post_state
        let created = output.created_address.unwrap();
        assert!(output.post_state.accounts.contains_key(&created));
    }

    /// Test execution builder pattern
    #[test]
    fn test_execution_builder() {
        let sender = Address::repeat_byte(0x01);

        let mut state = InMemoryDB::new();
        state.insert_account(
            sender,
            AccountState::new_with_balance(U256::from(10u128.pow(18))),
        );

        let (output, _) = ExecutionBuilder::new()
            .with_block(BlockEnv::testnet())
            .with_tx(TxInput::transfer(
                sender,
                Address::repeat_byte(0x02),
                U256::from(1000u64),
            ))
            .with_state(state)
            .execute()
            .unwrap();

        assert!(output.is_success());
    }

    /// Test simulation (dry-run)
    #[test]
    fn test_simulate() {
        let sender = Address::repeat_byte(0x01);
        let receiver = Address::repeat_byte(0x02);

        let mut pre_state = InMemoryDB::new();
        pre_state.insert_account(
            sender,
            AccountState::new_with_balance(U256::from(10u128.pow(18))),
        );

        let input = ExecutionInput::new(
            BlockEnv::default(),
            TxInput::transfer(sender, receiver, U256::from(1000u64)),
            pre_state,
        );

        let output = ShadowExecutor::simulate(input).unwrap();

        assert!(output.is_success());
        // Note: simulation doesn't commit, so post_state is pre_state
    }

    /// Test deterministic execution
    #[test]
    fn test_deterministic_execution() {
        let sender = Address::repeat_byte(0x01);
        let receiver = Address::repeat_byte(0x02);

        let create_input = || {
            let mut pre_state = InMemoryDB::new();
            pre_state.insert_account(
                sender,
                AccountState::new_with_balance(U256::from(10u128.pow(18))),
            );

            ExecutionInput::new(
                BlockEnv::default(),
                TxInput::transfer(sender, receiver, U256::from(1000u64)),
                pre_state,
            )
        };

        // Execute twice with identical inputs
        let (output1, commitment1) = ShadowExecutor::execute(create_input()).unwrap();
        let (output2, commitment2) = ShadowExecutor::execute(create_input()).unwrap();

        // Should produce identical commitments
        assert_eq!(commitment1.commitment, commitment2.commitment);
        assert_eq!(output1.gas_used, output2.gas_used);
        assert_eq!(output1.post_state_root(), output2.post_state_root());
    }
}
