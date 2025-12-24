// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.20;

/// @title RISC Zero Verifier Interface
/// @notice Interface for the RISC Zero Groth16 verifier
interface IRiscZeroVerifier {
    /// @notice Verify a Groth16 proof
    /// @param seal The proof seal
    /// @param imageId The image ID of the guest program
    /// @param journalDigest The SHA256 digest of the journal
    /// @return True if the proof is valid
    function verify(
        bytes calldata seal,
        bytes32 imageId,
        bytes32 journalDigest
    ) external view returns (bool);
}
