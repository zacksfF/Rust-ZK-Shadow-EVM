//! Execution input types for Shadow-EVM
//!
//! Defines the public and private inputs for ZK proof generation.
//! The hash of ExecutionInput becomes part of the ZK proof's public input.

use crate::hashing::hash_struct;
use crate::state::InMemoryDB;
use crate::types::{Address, BlockNumber, Gas, Hash, Timestamp, U256};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

extern crate alloc;

/// Block environment for execution context
///
/// Contains all block-level parameters needed for EVM execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEnv {
    /// Block number
    pub number: BlockNumber,
    /// Block timestamp (seconds since epoch)
    pub timestamp: Timestamp,
    /// Block gas limit
    pub gas_limit: Gas,
    /// Block coinbase (miner/validator address)
    pub coinbase: Address,
    /// Block base fee (EIP-1559)
    pub base_fee: U256,
    /// Previous block's RANDAO value (post-merge)
    pub prev_randao: Hash,
    /// Chain ID (1 for mainnet, etc.)
    pub chain_id: u64,
}

impl Default for BlockEnv {
    fn default() -> Self {
        Self {
            number: 1,
            timestamp: 1700000000, // Nov 2023
            gas_limit: 30_000_000,
            coinbase: Address::ZERO,
            base_fee: U256::from(1_000_000_000u64), // 1 gwei
            prev_randao: Hash::ZERO,
            chain_id: 1, // Mainnet
        }
    }
}

impl BlockEnv {
    /// Create a new block environment with custom parameters
    pub fn new(
        number: BlockNumber,
        timestamp: Timestamp,
        gas_limit: Gas,
        chain_id: u64,
    ) -> Self {
        Self {
            number,
            timestamp,
            gas_limit,
            chain_id,
            ..Default::default()
        }
    }

    /// Create a testnet block environment
    pub fn testnet() -> Self {
        Self {
            chain_id: 31337, // Local/Hardhat chain ID
            ..Default::default()
        }
    }
}

/// Transaction input for EVM execution
///
/// Contains all transaction parameters needed for a single EVM call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInput {
    /// Caller address (tx.origin and initial msg.sender)
    pub caller: Address,
    /// Target contract address (None for contract creation)
    pub to: Option<Address>,
    /// Value to transfer (in wei)
    pub value: U256,
    /// Input data (calldata)
    pub data: Vec<u8>,
    /// Gas limit for execution
    pub gas_limit: Gas,
    /// Gas price (for gas accounting)
    pub gas_price: U256,
    /// Transaction nonce
    pub nonce: u64,
}

impl Default for TxInput {
    fn default() -> Self {
        Self {
            caller: Address::ZERO,
            to: None,
            value: U256::ZERO,
            data: Vec::new(),
            gas_limit: 10_000_000,
            gas_price: U256::from(1_000_000_000u64), // 1 gwei
            nonce: 0,
        }
    }
}

impl TxInput {
    /// Create a simple call transaction
    pub fn call(caller: Address, to: Address, data: Vec<u8>) -> Self {
        Self {
            caller,
            to: Some(to),
            data,
            ..Default::default()
        }
    }

    /// Create a value transfer transaction
    pub fn transfer(caller: Address, to: Address, value: U256) -> Self {
        Self {
            caller,
            to: Some(to),
            value,
            gas_limit: 21_000, // Standard transfer gas
            ..Default::default()
        }
    }

    /// Create a contract creation transaction
    pub fn create(caller: Address, init_code: Vec<u8>, value: U256) -> Self {
        Self {
            caller,
            to: None,
            data: init_code,
            value,
            ..Default::default()
        }
    }

    /// Check if this is a contract creation
    pub fn is_create(&self) -> bool {
        self.to.is_none()
    }

    /// Set the gas limit
    pub fn with_gas_limit(mut self, gas_limit: Gas) -> Self {
        self.gas_limit = gas_limit;
        self
    }

    /// Set the nonce
    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }
}

/// Complete execution input for Shadow-EVM
///
/// This struct contains everything needed to reproduce EVM execution deterministically.
/// The hash of this struct is part of the ZK proof's public input.
///
/// # ZK Considerations
/// - All fields must be deterministically serializable
/// - The hash binds the execution to specific inputs
/// - Pre-state root allows verification of initial state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionInput {
    /// Block environment
    pub block: BlockEnv,
    /// Transaction input
    pub tx: TxInput,
    /// Pre-execution state
    pub pre_state: InMemoryDB,
}

impl ExecutionInput {
    /// Create new execution input
    pub fn new(block: BlockEnv, tx: TxInput, pre_state: InMemoryDB) -> Self {
        Self {
            block,
            tx,
            pre_state,
        }
    }

    /// Compute deterministic hash of this input
    ///
    /// This hash is used as part of the ZK proof's public input.
    pub fn hash(&self) -> Hash {
        hash_struct(self)
    }

    /// Get the pre-state root
    pub fn pre_state_root(&self) -> Hash {
        self.pre_state.compute_state_root()
    }

    /// Get the caller address
    pub fn caller(&self) -> Address {
        self.tx.caller
    }

    /// Get the target address (if any)
    pub fn target(&self) -> Option<Address> {
        self.tx.to
    }

    /// Check if this is a contract creation
    pub fn is_create(&self) -> bool {
        self.tx.is_create()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AccountState;

    #[test]
    fn test_block_env_default() {
        let block = BlockEnv::default();
        assert_eq!(block.chain_id, 1);
        assert_eq!(block.gas_limit, 30_000_000);
    }

    #[test]
    fn test_tx_call() {
        let caller = Address::repeat_byte(0x01);
        let to = Address::repeat_byte(0x02);
        let data = vec![0x12, 0x34, 0x56, 0x78];

        let tx = TxInput::call(caller, to, data.clone());

        assert_eq!(tx.caller, caller);
        assert_eq!(tx.to, Some(to));
        assert_eq!(tx.data, data);
        assert!(!tx.is_create());
    }

    #[test]
    fn test_tx_transfer() {
        let caller = Address::repeat_byte(0x01);
        let to = Address::repeat_byte(0x02);
        let value = U256::from(1_000_000_000_000_000_000u128);

        let tx = TxInput::transfer(caller, to, value);

        assert_eq!(tx.value, value);
        assert_eq!(tx.gas_limit, 21_000);
    }

    #[test]
    fn test_tx_create() {
        let caller = Address::repeat_byte(0x01);
        let init_code = vec![0x60, 0x00, 0x60, 0x00, 0xf3];

        let tx = TxInput::create(caller, init_code.clone(), U256::ZERO);

        assert!(tx.is_create());
        assert!(tx.to.is_none());
        assert_eq!(tx.data, init_code);
    }

    #[test]
    fn test_execution_input_hash_deterministic() {
        let mut pre_state = InMemoryDB::new();
        pre_state.insert_account(
            Address::repeat_byte(0x01),
            AccountState::new_with_balance(U256::from(1000u64)),
        );

        let input1 = ExecutionInput::new(
            BlockEnv::default(),
            TxInput::default(),
            pre_state.clone(),
        );

        let input2 = ExecutionInput::new(BlockEnv::default(), TxInput::default(), pre_state);

        assert_eq!(input1.hash(), input2.hash());
    }

    #[test]
    fn test_execution_input_hash_different() {
        let pre_state = InMemoryDB::new();

        let input1 = ExecutionInput::new(
            BlockEnv::new(1, 1000, 30_000_000, 1),
            TxInput::default(),
            pre_state.clone(),
        );

        let input2 = ExecutionInput::new(
            BlockEnv::new(2, 1000, 30_000_000, 1), // Different block number
            TxInput::default(),
            pre_state,
        );

        assert_ne!(input1.hash(), input2.hash());
    }
}
