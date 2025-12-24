//! EVM execution module for ZKVM guest
//!
//! This module provides the EVM execution logic that runs inside the RISC-V ZK-VM.

pub mod executor;
pub mod host_io;
pub mod state;

// Re-exports
pub use executor::ZkExecutor;
pub use host_io::{commit_output, read_input};
