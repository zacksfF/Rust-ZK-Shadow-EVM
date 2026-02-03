# Shadow-EVM Threat Model

This document outlines the security considerations and potential attack vectors for Shadow-EVM.

## Trust Assumptions

| Entity | Trust Level | Responsibility |
|--------|-------------|----------------|
| RISC Zero Prover | Trusted | Correct proof generation |
| Guest Code | Verified | Deterministic execution |
| On-Chain Verifier | Trustless | Proof verification |
| Host (Prover) | Untrusted | Input preparation |

## Threat Categories

### 1. Malicious Prover

**Attack**: Prover submits invalid proofs claiming false execution results.

**Mitigation**: 
- RISC Zero Groth16 proofs are cryptographically sound
- On-chain verifier rejects invalid proofs
- Image ID binding ensures only authorized guest code is accepted

### 2. Replayed Commitments

**Attack**: Attacker replays a valid proof from a previous execution.

**Mitigation**:
- Commitments include transaction hash and block number
- Nonce tracking in the verifier contract
- State root chaining prevents reordering

### 3. Input Tampering

**Attack**: Host provides incorrect pre-state or block parameters.

**Mitigation**:
- Pre-state root is committed in the proof
- Verifier checks pre-state matches known chain state
- Block parameters can be verified against on-chain data

### 4. Guest/Host Version Skew

**Attack**: Host uses different guest image than verifier expects.

**Mitigation**:
- Image ID embedded in Solidity verifier
- Runtime checks compare expected vs actual image ID
- Version binding in deployment scripts

### 5. Chain Reorganization

**Attack**: Transaction proven against a block that gets reorged.

**Mitigation**:
- Wait for sufficient confirmations before proving
- Include block hash in commitment
- Application-level finality requirements

## Security Invariants

1. **Computational Integrity**: Proof valid ⟹ execution was correct
2. **State Binding**: `pre_state + tx → post_state` is cryptographically committed
3. **Non-Forgery**: Cannot create valid proof without running guest code
4. **Determinism**: Same inputs always produce same outputs

## Known Limitations

- **Side Channels**: Cycle count may leak execution complexity
- **Memory Limits**: Large state may exceed guest memory
- **Gas Semantics**: EVM gas is tracked but not enforced by ZK circuit

## Incident Response

1. Pause verifier contract if vulnerability discovered
2. Deploy new guest with patched image ID
3. Update verifier with new image ID
4. Resume operations after security review
