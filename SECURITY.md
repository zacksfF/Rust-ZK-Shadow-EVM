# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it by:

1. **Do NOT open a public issue**
2. Send an email to the repository maintainer
3. Include detailed steps to reproduce
4. Allow time for a fix before public disclosure

## Security Model

Shadow-EVM provides the following security guarantees:

### Cryptographic Security

- **STARK Proofs**: Computational integrity of EVM execution
- **Groth16 SNARKs**: Constant-size proofs for on-chain verification
- **Keccak256 Commitments**: Bind execution input to output

### What We Guarantee

| Property | Guarantee |
|----------|-----------|
| Execution Correctness | EVM executed correctly inside ZK-VM |
| State Transition | Pre-state + tx produces post-state |
| Determinism | Same input always produces same output |

### What We Do NOT Guarantee

| Property | Reason |
|----------|--------|
| Input Validity | Caller must validate inputs |
| Pre-State Existence | Must verify pre-state exists on-chain |
| Economic Security | Separate from cryptographic guarantees |

## Dependencies

We rely on:
- [RISC Zero](https://risczero.com/) - Trusted setup and ZK-VM
- [revm](https://github.com/bluealloy/revm) - EVM implementation
- [alloy](https://github.com/alloy-rs/alloy) - Ethereum primitives

## Audit Status

This project has not yet been audited. Use at your own risk.

