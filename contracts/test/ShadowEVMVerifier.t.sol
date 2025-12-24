// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/ShadowEVMVerifier.sol";
import "../src/IRiscZeroVerifier.sol";

/// @title Mock RISC Zero Verifier for testing
contract MockRiscZeroVerifier is IRiscZeroVerifier {
    bool public shouldVerify = true;
    
    function setVerifyResult(bool _result) external {
        shouldVerify = _result;
    }
    
    function verify(
        bytes calldata,
        bytes32,
        bytes32
    ) external view override returns (bool) {
        return shouldVerify;
    }
}

/// @title Shadow-EVM Verifier Tests
contract ShadowEVMVerifierTest is Test {
    ShadowEVMVerifier public verifier;
    MockRiscZeroVerifier public mockVerifier;
    
    bytes32 constant IMAGE_ID = bytes32(uint256(0x1234));
    
    // Sample commitment values
    bytes32 constant INPUT_HASH = bytes32(uint256(0x1111));
    bytes32 constant OUTPUT_HASH = bytes32(uint256(0x2222));
    bytes32 constant PRE_STATE_ROOT = bytes32(uint256(0x3333));
    bytes32 constant POST_STATE_ROOT = bytes32(uint256(0x4444));
    bytes32 constant COMMITMENT = bytes32(uint256(0x5555));
    
    function setUp() public {
        mockVerifier = new MockRiscZeroVerifier();
        verifier = new ShadowEVMVerifier(address(mockVerifier), IMAGE_ID);
    }
    
    function _createJournal() internal pure returns (bytes memory) {
        return abi.encodePacked(
            INPUT_HASH,
            OUTPUT_HASH,
            PRE_STATE_ROOT,
            POST_STATE_ROOT,
            COMMITMENT
        );
    }
    
    function test_Constructor() public view {
        assertEq(address(verifier.verifier()), address(mockVerifier));
        assertEq(verifier.imageId(), IMAGE_ID);
    }
    
    function test_VerifySuccess() public {
        bytes memory seal = hex"1234";
        bytes memory journal = _createJournal();
        
        ShadowEVMVerifier.ExecutionCommitment memory commitment = verifier.verify(seal, journal);
        
        assertEq(commitment.inputHash, INPUT_HASH);
        assertEq(commitment.outputHash, OUTPUT_HASH);
        assertEq(commitment.preStateRoot, PRE_STATE_ROOT);
        assertEq(commitment.postStateRoot, POST_STATE_ROOT);
        assertEq(commitment.commitment, COMMITMENT);
        
        assertTrue(verifier.isVerified(COMMITMENT));
        assertEq(verifier.getPostStateRoot(PRE_STATE_ROOT), POST_STATE_ROOT);
    }
    
    function test_VerifyEmitsEvents() public {
        bytes memory seal = hex"1234";
        bytes memory journal = _createJournal();
        
        vm.expectEmit(true, true, true, true);
        emit ShadowEVMVerifier.ExecutionVerified(
            COMMITMENT,
            PRE_STATE_ROOT,
            POST_STATE_ROOT,
            address(this)
        );
        
        vm.expectEmit(true, true, true, true);
        emit ShadowEVMVerifier.StateTransitionRecorded(
            PRE_STATE_ROOT,
            POST_STATE_ROOT,
            COMMITMENT
        );
        
        verifier.verify(seal, journal);
    }
    
    function test_VerifyFailsOnInvalidProof() public {
        mockVerifier.setVerifyResult(false);
        
        bytes memory seal = hex"1234";
        bytes memory journal = _createJournal();
        
        vm.expectRevert(ShadowEVMVerifier.ProofVerificationFailed.selector);
        verifier.verify(seal, journal);
    }
    
    function test_VerifyFailsOnDuplicateCommitment() public {
        bytes memory seal = hex"1234";
        bytes memory journal = _createJournal();
        
        // First verification should succeed
        verifier.verify(seal, journal);
        
        // Second verification should fail
        vm.expectRevert(abi.encodeWithSelector(
            ShadowEVMVerifier.CommitmentAlreadyVerified.selector,
            COMMITMENT
        ));
        verifier.verify(seal, journal);
    }
    
    function test_VerifyFailsOnInvalidJournal() public {
        bytes memory seal = hex"1234";
        bytes memory journal = hex"1234"; // Too short
        
        vm.expectRevert(ShadowEVMVerifier.JournalDecodingFailed.selector);
        verifier.verify(seal, journal);
    }
    
    function test_IsVerified() public {
        assertFalse(verifier.isVerified(COMMITMENT));
        
        bytes memory seal = hex"1234";
        bytes memory journal = _createJournal();
        verifier.verify(seal, journal);
        
        assertTrue(verifier.isVerified(COMMITMENT));
    }
    
    function test_VerifyStateChain() public {
        // Create a chain of state transitions
        bytes32 root1 = bytes32(uint256(1));
        bytes32 root2 = bytes32(uint256(2));
        bytes32 root3 = bytes32(uint256(3));
        
        // Verify first transition (root1 -> root2)
        bytes memory journal1 = abi.encodePacked(
            INPUT_HASH,
            OUTPUT_HASH,
            root1,
            root2,
            bytes32(uint256(0xA))
        );
        verifier.verify(hex"1234", journal1);
        
        // Verify second transition (root2 -> root3)
        bytes memory journal2 = abi.encodePacked(
            INPUT_HASH,
            OUTPUT_HASH,
            root2,
            root3,
            bytes32(uint256(0xB))
        );
        verifier.verify(hex"1234", journal2);
        
        // Verify the chain
        bytes32[] memory chain = new bytes32[](3);
        chain[0] = root1;
        chain[1] = root2;
        chain[2] = root3;
        
        assertTrue(verifier.verifyStateChain(chain));
        
        // Invalid chain
        bytes32[] memory invalidChain = new bytes32[](3);
        invalidChain[0] = root1;
        invalidChain[1] = root3; // Skipped root2
        invalidChain[2] = root2;
        
        assertFalse(verifier.verifyStateChain(invalidChain));
    }
    
    function test_BatchVerify() public {
        bytes[] memory seals = new bytes[](2);
        seals[0] = hex"1234";
        seals[1] = hex"5678";
        
        bytes[] memory journals = new bytes[](2);
        journals[0] = abi.encodePacked(
            INPUT_HASH,
            OUTPUT_HASH,
            PRE_STATE_ROOT,
            POST_STATE_ROOT,
            bytes32(uint256(0xAAAA))
        );
        journals[1] = abi.encodePacked(
            INPUT_HASH,
            OUTPUT_HASH,
            POST_STATE_ROOT, // Chain continues
            bytes32(uint256(0x6666)),
            bytes32(uint256(0xBBBB))
        );
        
        ShadowEVMVerifier.ExecutionCommitment[] memory commitments = 
            verifier.batchVerify(seals, journals);
        
        assertEq(commitments.length, 2);
        assertEq(commitments[0].commitment, bytes32(uint256(0xAAAA)));
        assertEq(commitments[1].commitment, bytes32(uint256(0xBBBB)));
    }
}
