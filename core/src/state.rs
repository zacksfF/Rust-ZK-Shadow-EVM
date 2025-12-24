//! In-memory state database for Shadow-EVM
//!
//! Implements `revm::Database` for EVM state access during execution.
//! Uses a simplified HashMap-based model (not full Merkle Patricia Trie).

use crate::errors::{Result, ShadowEvmError};
use crate::hashing::keccak256;
use crate::types::{Address, Hash, U256};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use revm::primitives::{AccountInfo, Bytecode, KECCAK_EMPTY};
use revm::Database;
use serde::{Deserialize, Serialize};

extern crate alloc;

/// Account state representation
///
/// Contains all account data needed for EVM execution:
/// - Balance in wei
/// - Transaction nonce
/// - Contract code and its hash
/// - Storage slots
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountState {
    /// Account balance in wei
    pub balance: U256,
    /// Transaction nonce
    pub nonce: u64,
    /// Hash of contract code (KECCAK_EMPTY if EOA)
    pub code_hash: Hash,
    /// Contract bytecode (empty for EOA)
    pub code: Vec<u8>,
    /// Storage slots (key -> value)
    pub storage: BTreeMap<U256, U256>,
}

impl AccountState {
    /// Create a new externally owned account (EOA) with balance
    pub fn new_with_balance(balance: U256) -> Self {
        Self {
            balance,
            nonce: 0,
            code_hash: KECCAK_EMPTY,
            code: Vec::new(),
            storage: BTreeMap::new(),
        }
    }

    /// Create a contract account with code
    pub fn new_contract(code: Vec<u8>, balance: U256) -> Self {
        let code_hash = keccak256(&code);
        Self {
            balance,
            nonce: 1, // Contracts start with nonce 1
            code_hash,
            code,
            storage: BTreeMap::new(),
        }
    }

    /// Create a contract account with code and storage
    pub fn new_contract_with_storage(
        code: Vec<u8>,
        balance: U256,
        storage: BTreeMap<U256, U256>,
    ) -> Self {
        let code_hash = keccak256(&code);
        Self {
            balance,
            nonce: 1,
            code_hash,
            code,
            storage,
        }
    }

    /// Check if this is a contract account
    pub fn is_contract(&self) -> bool {
        !self.code.is_empty()
    }

    /// Check if this is an empty account (can be pruned)
    pub fn is_empty(&self) -> bool {
        self.balance.is_zero() && self.nonce == 0 && self.code.is_empty()
    }

    /// Set a storage slot value
    pub fn set_storage(&mut self, slot: U256, value: U256) {
        if value.is_zero() {
            self.storage.remove(&slot);
        } else {
            self.storage.insert(slot, value);
        }
    }

    /// Get a storage slot value
    pub fn get_storage(&self, slot: &U256) -> U256 {
        self.storage.get(slot).copied().unwrap_or(U256::ZERO)
    }
}

/// In-memory state database
///
/// Implements `revm::Database` for EVM execution.
/// Uses BTreeMap for deterministic iteration order (important for ZK proofs).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InMemoryDB {
    /// Account states indexed by address
    pub accounts: BTreeMap<Address, AccountState>,
    /// Block hashes for BLOCKHASH opcode (block number -> hash)
    pub block_hashes: BTreeMap<u64, Hash>,
}

impl InMemoryDB {
    /// Create a new empty database
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert an account into the database
    pub fn insert_account(&mut self, address: Address, account: AccountState) {
        self.accounts.insert(address, account);
    }

    /// Get an account reference
    pub fn get_account(&self, address: &Address) -> Option<&AccountState> {
        self.accounts.get(address)
    }

    /// Get an account (mutable)
    pub fn get_account_mut(&mut self, address: &Address) -> Option<&mut AccountState> {
        self.accounts.get_mut(address)
    }

    /// Remove an account from the database
    pub fn remove_account(&mut self, address: &Address) -> Option<AccountState> {
        self.accounts.remove(address)
    }

    /// Check if an account exists
    pub fn account_exists(&self, address: &Address) -> bool {
        self.accounts.contains_key(address)
    }

    /// Insert a block hash (for BLOCKHASH opcode)
    pub fn insert_block_hash(&mut self, number: u64, hash: Hash) {
        self.block_hashes.insert(number, hash);
    }

    /// Compute state root commitment (simplified - not MPT)
    ///
    /// For v0.1, we use a deterministic hash of all accounts.
    /// This is NOT Ethereum-compatible but is sufficient for ZK proofs.
    ///
    /// Future versions will implement full Merkle Patricia Trie.
    pub fn compute_state_root(&self) -> Hash {
        use crate::hashing::hash_struct;
        hash_struct(&self.accounts)
    }

    /// Get the number of accounts in the database
    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }

    /// Clear all accounts
    pub fn clear(&mut self) {
        self.accounts.clear();
        self.block_hashes.clear();
    }

    /// Merge changes from another database
    ///
    /// Used after execution to apply state changes.
    pub fn merge(&mut self, other: &InMemoryDB) {
        for (address, account) in &other.accounts {
            self.accounts.insert(*address, account.clone());
        }
    }
}

impl Database for InMemoryDB {
    type Error = ShadowEvmError;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>> {
        Ok(self.accounts.get(&address).map(|acc| AccountInfo {
            balance: acc.balance,
            nonce: acc.nonce,
            code_hash: acc.code_hash,
            code: if acc.code.is_empty() {
                None
            } else {
                Some(Bytecode::new_raw(acc.code.clone().into()))
            },
        }))
    }

    fn code_by_hash(&mut self, code_hash: Hash) -> Result<Bytecode> {
        // Handle empty code case
        if code_hash == KECCAK_EMPTY {
            return Ok(Bytecode::default());
        }

        // Linear search - acceptable for small state in v0.1
        for acc in self.accounts.values() {
            if acc.code_hash == code_hash && !acc.code.is_empty() {
                return Ok(Bytecode::new_raw(acc.code.clone().into()));
            }
        }

        // Return empty bytecode if not found
        Ok(Bytecode::default())
    }

    fn storage(&mut self, address: Address, slot: U256) -> Result<U256> {
        Ok(self
            .accounts
            .get(&address)
            .map(|acc| acc.get_storage(&slot))
            .unwrap_or(U256::ZERO))
    }

    fn block_hash(&mut self, number: u64) -> Result<Hash> {
        Ok(self.block_hashes.get(&number).copied().unwrap_or(Hash::ZERO))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_eoa() {
        let balance = U256::from(1_000_000_000_000_000_000u128); // 1 ETH
        let account = AccountState::new_with_balance(balance);

        assert_eq!(account.balance, balance);
        assert_eq!(account.nonce, 0);
        assert!(!account.is_contract());
        assert!(!account.is_empty());
    }

    #[test]
    fn test_new_contract() {
        let code = vec![0x60, 0x00, 0x60, 0x00, 0xf3]; // PUSH1 0, PUSH1 0, RETURN
        let account = AccountState::new_contract(code.clone(), U256::ZERO);

        assert!(account.is_contract());
        assert_eq!(account.nonce, 1);
        assert_eq!(account.code, code);
        assert_ne!(account.code_hash, KECCAK_EMPTY);
    }

    #[test]
    fn test_storage_operations() {
        let mut account = AccountState::new_with_balance(U256::ZERO);

        let slot = U256::from(1u64);
        let value = U256::from(42u64);

        // Initially zero
        assert_eq!(account.get_storage(&slot), U256::ZERO);

        // Set value
        account.set_storage(slot, value);
        assert_eq!(account.get_storage(&slot), value);

        // Set to zero removes slot
        account.set_storage(slot, U256::ZERO);
        assert!(!account.storage.contains_key(&slot));
    }

    #[test]
    fn test_database_basic() {
        let mut db = InMemoryDB::new();
        let addr = Address::repeat_byte(0x01);
        let balance = U256::from(1000u64);

        db.insert_account(addr, AccountState::new_with_balance(balance));

        let info = db.basic(addr).unwrap().unwrap();
        assert_eq!(info.balance, balance);
        assert_eq!(info.nonce, 0);
    }

    #[test]
    fn test_database_storage() {
        let mut db = InMemoryDB::new();
        let addr = Address::repeat_byte(0x01);

        let mut account = AccountState::new_with_balance(U256::ZERO);
        account.set_storage(U256::from(1u64), U256::from(42u64));
        db.insert_account(addr, account);

        let value = db.storage(addr, U256::from(1u64)).unwrap();
        assert_eq!(value, U256::from(42u64));

        // Non-existent slot returns zero
        let empty = db.storage(addr, U256::from(999u64)).unwrap();
        assert_eq!(empty, U256::ZERO);
    }

    #[test]
    fn test_state_root_deterministic() {
        let mut db1 = InMemoryDB::new();
        let mut db2 = InMemoryDB::new();

        let addr = Address::repeat_byte(0x01);
        let account = AccountState::new_with_balance(U256::from(1000u64));

        db1.insert_account(addr, account.clone());
        db2.insert_account(addr, account);

        assert_eq!(db1.compute_state_root(), db2.compute_state_root());
    }

    #[test]
    fn test_block_hash() {
        let mut db = InMemoryDB::new();

        let hash = Hash::repeat_byte(0xAB);
        db.insert_block_hash(100, hash);

        assert_eq!(db.block_hash(100).unwrap(), hash);
        assert_eq!(db.block_hash(101).unwrap(), Hash::ZERO);
    }
}
