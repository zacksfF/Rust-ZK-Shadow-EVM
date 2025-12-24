// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.20;

import {IRiscZeroVerifier} from "./IRiscZeroVerifier.sol";

/// @title Shadow-EVM Verifier
/// @notice On-chain verifier for Shadow-EVM ZK proofs
/// @dev Uses RISC Zero's Groth16 verifier for proof verification
contract ShadowEVMVerifier {
    /// @notice The RISC Zero verifier contract
    IRiscZeroVerifier public immutable verifier;
    
    /// @notice The image ID of the Shadow-EVM guest program
    bytes32 public immutable imageId;
    
    /// @notice Mapping of verified commitments
    mapping(bytes32 => bool) public verifiedCommitments;
    
    /// @notice Mapping of verified state transitions
    mapping(bytes32 => bytes32) public stateTransitions;
    
    /// @notice Emitted when an execution is verified
    event ExecutionVerified(
        bytes32 indexed commitment,
        bytes32 indexed preStateRoot,
        bytes32 indexed postStateRoot,
        address verifier
    );
    
    /// @notice Emitted when a state transition is recorded
    event StateTransitionRecorded(
        bytes32 indexed preStateRoot,
        bytes32 indexed postStateRoot,
        bytes32 commitment
    );
    
    /// @notice Error thrown when proof verification fails
    error ProofVerificationFailed();
    
    /// @notice Error thrown when commitment is already verified
    error CommitmentAlreadyVerified(bytes32 commitment);
    
    /// @notice Error thrown when journal decoding fails
    error JournalDecodingFailed();
    
    /// @notice Decoded execution commitment from journal
    struct ExecutionCommitment {
        bytes32 inputHash;
        bytes32 outputHash;
        bytes32 preStateRoot;
        bytes32 postStateRoot;
        bytes32 commitment;
    }
    
    /// @notice Constructor
    /// @param _verifier Address of the RISC Zero verifier contract
    /// @param _imageId The image ID of the Shadow-EVM guest program
    constructor(address _verifier, bytes32 _imageId) {
        verifier = IRiscZeroVerifier(_verifier);
        imageId = _imageId;
    }
    
    /// @notice Verify a Shadow-EVM execution proof
    /// @param seal The proof seal (Groth16 proof)
    /// @param journal The journal containing the execution commitment
    /// @return commitment The verified execution commitment
    function verify(
        bytes calldata seal,
        bytes calldata journal
    ) external returns (ExecutionCommitment memory commitment) {
        // Verify the proof using RISC Zero verifier
        bool verified = verifier.verify(seal, imageId, sha256(journal));
        if (!verified) {
            revert ProofVerificationFailed();
        }
        
        // Decode the commitment from the journal
        commitment = _decodeCommitment(journal);
        
        // Check if already verified
        if (verifiedCommitments[commitment.commitment]) {
            revert CommitmentAlreadyVerified(commitment.commitment);
        }
        
        // Mark as verified
        verifiedCommitments[commitment.commitment] = true;
        
        // Record state transition
        stateTransitions[commitment.preStateRoot] = commitment.postStateRoot;
        
        // Emit events
        emit ExecutionVerified(
            commitment.commitment,
            commitment.preStateRoot,
            commitment.postStateRoot,
            msg.sender
        );
        
        emit StateTransitionRecorded(
            commitment.preStateRoot,
            commitment.postStateRoot,
            commitment.commitment
        );
        
        return commitment;
    }
    
    /// @notice Check if a commitment has been verified
    /// @param commitment The commitment to check
    /// @return True if the commitment has been verified
    function isVerified(bytes32 commitment) external view returns (bool) {
        return verifiedCommitments[commitment];
    }
    
    /// @notice Get the post-state root for a given pre-state root
    /// @param preStateRoot The pre-state root
    /// @return The post-state root (or zero if not found)
    function getPostStateRoot(bytes32 preStateRoot) external view returns (bytes32) {
        return stateTransitions[preStateRoot];
    }
    
    /// @notice Verify a state transition chain
    /// @param roots Array of state roots (must be consecutive transitions)
    /// @return True if all transitions are verified
    function verifyStateChain(bytes32[] calldata roots) external view returns (bool) {
        if (roots.length < 2) {
            return true;
        }
        
        for (uint256 i = 0; i < roots.length - 1; i++) {
            if (stateTransitions[roots[i]] != roots[i + 1]) {
                return false;
            }
        }
        
        return true;
    }
    
    /// @notice Decode commitment from journal bytes
    /// @param journal The journal bytes
    /// @return commitment The decoded commitment
    function _decodeCommitment(
        bytes calldata journal
    ) internal pure returns (ExecutionCommitment memory commitment) {
        // Journal format: 5 x 32-byte hashes = 160 bytes
        if (journal.length < 160) {
            revert JournalDecodingFailed();
        }
        
        commitment.inputHash = bytes32(journal[0:32]);
        commitment.outputHash = bytes32(journal[32:64]);
        commitment.preStateRoot = bytes32(journal[64:96]);
        commitment.postStateRoot = bytes32(journal[96:128]);
        commitment.commitment = bytes32(journal[128:160]);
    }
    
    /// @notice Batch verify multiple proofs
    /// @param seals Array of proof seals
    /// @param journals Array of journals
    /// @return commitments Array of verified commitments
    function batchVerify(
        bytes[] calldata seals,
        bytes[] calldata journals
    ) external returns (ExecutionCommitment[] memory commitments) {
        require(seals.length == journals.length, "Length mismatch");
        
        commitments = new ExecutionCommitment[](seals.length);
        
        for (uint256 i = 0; i < seals.length; i++) {
            commitments[i] = this.verify(seals[i], journals[i]);
        }
        
        return commitments;
    }
}
