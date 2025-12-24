//! ZK-compatible state wrapper
//!
//! Provides a wrapper around the core InMemoryDB that ensures
//! all state operations are deterministic and ZK-friendly.

use shadow_evm_core::prelude::*;

/// ZK-compatible state wrapper
///
/// This wrapper ensures all state operations are deterministic
/// and suitable for execution inside a ZK-VM.
pub struct ZkState {
    /// The underlying in-memory database
    db: InMemoryDB,
}

impl ZkState {
    /// Create a new ZK state from an InMemoryDB
    pub fn new(db: InMemoryDB) -> Self {
        Self { db }
    }

    /// Get the underlying database
    pub fn into_db(self) -> InMemoryDB {
        self.db
    }

    /// Get a reference to the underlying database
    pub fn db(&self) -> &InMemoryDB {
        &self.db
    }

    /// Get a mutable reference to the underlying database
    pub fn db_mut(&mut self) -> &mut InMemoryDB {
        &mut self.db
    }

    /// Compute the state root
    pub fn state_root(&self) -> Hash {
        self.db.compute_state_root()
    }

    /// Verify the state root matches expected value
    pub fn verify_root(&self, expected: &Hash) -> bool {
        self.state_root() == *expected
    }
}

impl From<InMemoryDB> for ZkState {
    fn from(db: InMemoryDB) -> Self {
        Self::new(db)
    }
}

impl From<ZkState> for InMemoryDB {
    fn from(state: ZkState) -> Self {
        state.into_db()
    }
}
