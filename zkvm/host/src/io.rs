//! I/O utilities for the host
//!
//! Handles serialization/deserialization of inputs and outputs
//! for communication with the guest and external systems.

use anyhow::{Context, Result};
use shadow_evm_core::prelude::*;
use std::fs;
use std::path::Path;

/// Serialize ExecutionInput to bytes
pub fn serialize_input(input: &ExecutionInput) -> Result<Vec<u8>> {
    bincode::serialize(input).context("Failed to serialize ExecutionInput")
}

/// Deserialize ExecutionInput from bytes
pub fn deserialize_input(data: &[u8]) -> Result<ExecutionInput> {
    bincode::deserialize(data).context("Failed to deserialize ExecutionInput")
}

/// Serialize ExecutionCommitment to bytes
pub fn serialize_commitment(commitment: &ExecutionCommitment) -> Result<Vec<u8>> {
    bincode::serialize(commitment).context("Failed to serialize ExecutionCommitment")
}

/// Deserialize ExecutionCommitment from bytes
pub fn deserialize_commitment(data: &[u8]) -> Result<ExecutionCommitment> {
    bincode::deserialize(data).context("Failed to deserialize ExecutionCommitment")
}

/// Save ExecutionInput to a JSON file
pub fn save_input_json<P: AsRef<Path>>(input: &ExecutionInput, path: P) -> Result<()> {
    let json = serde_json::to_string_pretty(input)?;
    fs::write(path, json)?;
    Ok(())
}

/// Load ExecutionInput from a JSON file
pub fn load_input_json<P: AsRef<Path>>(path: P) -> Result<ExecutionInput> {
    let json = fs::read_to_string(path)?;
    let input: ExecutionInput = serde_json::from_str(&json)?;
    Ok(input)
}

/// Save binary data to a file
pub fn save_bytes<P: AsRef<Path>>(data: &[u8], path: P) -> Result<()> {
    fs::write(path, data)?;
    Ok(())
}

/// Load binary data from a file
pub fn load_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let data = fs::read(path)?;
    Ok(data)
}

/// Format a hash as a hex string
pub fn format_hash(hash: &Hash) -> String {
    format!("0x{}", hex::encode(hash.as_slice()))
}

/// Format commitment for display
pub fn format_commitment(commitment: &ExecutionCommitment) -> String {
    format!(
        "ExecutionCommitment {{\n  \
         input_hash: {},\n  \
         output_hash: {},\n  \
         pre_state_root: {},\n  \
         post_state_root: {},\n  \
         commitment: {}\n\
         }}",
        format_hash(&commitment.input_hash),
        format_hash(&commitment.output_hash),
        format_hash(&commitment.pre_state_root),
        format_hash(&commitment.post_state_root),
        format_hash(&commitment.commitment),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_roundtrip() {
        let input = ExecutionInput::new(BlockEnv::default(), TxInput::default(), InMemoryDB::new());

        let bytes = serialize_input(&input).unwrap();
        let decoded = deserialize_input(&bytes).unwrap();

        assert_eq!(input.hash(), decoded.hash());
    }
}
