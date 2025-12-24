# Rust-ZK-Shadow-EVM

**Zero-Knowledge Coprocessor for Verifiable Off-Chain Ethereum Execution**

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL%203.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-47%20passing-brightgreen)]()

---

## Overview

Shadow-EVM enables complex Ethereum computations to be executed **off-chain** while maintaining **on-chain security guarantees**. By embedding `revm` (Rust EVM) inside a RISC-V ZK-VM (RISC Zero), developers can prove the correct execution of Solidity bytecode without running it on Ethereum's expensive Layer 1.

### The Problem: Gas Ceiling

Traditional smart contracts are limited by block gas limits (~30M gas). Complex computations like machine learning inference, large-scale data processing, and batch transaction processing are often impossible or prohibitively expensive on-chain.

### The Solution: ZK Coprocessor

```
                              OFF-CHAIN
  ┌────────────────────────────────────────────────────────────┐
  │                                                            │
  │   ExecutionInput ──> RISC Zero ZK-VM ──> ZK Proof         │
  │   (Block + Tx + State)   (revm inside)    (~256 bytes)    │
  │                                                            │
  │   Cost: $0 gas, unlimited computation                     │
  └─────────────────────────┬──────────────────────────────────┘
                            │
                      ZK Proof
                            │
  ┌─────────────────────────▼──────────────────────────────────┐
  │                                                            │
  │   ShadowEVMVerifier.verify(proof) ──> State Transition    │
  │                                                            │
  │   Cost: ~230k gas (constant, regardless of computation)   │
  │                                                            │
  └────────────────────────────────────────────────────────────┘
                              ON-CHAIN
```

---

## Quick Start

### Prerequisites

| Tool | Version | Installation |
|------|---------|--------------|
| Rust | 1.75+ | [rustup.rs](https://rustup.rs/) |
| RISC Zero | 1.2+ | See below |
| Foundry | Latest | [getfoundry.sh](https://getfoundry.sh/) |

### Installation

```bash
git clone https://github.com/zacksfF/Rust-ZK-Shadow-EVM.git
cd Rust-ZK-Shadow-EVM

# Install RISC Zero toolchain (optional, for proof generation)
curl -L https://risczero.com/install | bash
rzup install

# Build
cargo build --workspace --release
```

### Run Tests

```bash
cargo test -p shadow-evm-core    # 38 Rust tests
cd contracts && forge test        # 9 Solidity tests
```

---

## Examples

### Simple ETH Transfer

**Source:** [examples/simple-call/main.rs](examples/simple-call/main.rs)

```bash
cd examples/simple-call && cargo run
```

```
Shadow-EVM: Simple Transfer

Sender:   0x0101010101010101010101010101010101010101
Receiver: 0x0202020202020202020202020202020202020202
Amount:   0.1 ETH

Pre-state root:  0xca5a15d74fe0fdbb894c03d5350cdfc...

--- Execution Result ---
Status:          Success
Gas used:        21000
Post-state root: 0xbe2111ce299666736115301379996ca...

--- Final Balances ---
Sender:   899979000000000000 wei
Receiver: 100000000000000000 wei

--- ZK Commitment ---
Commitment: 0x34c7c7c85ca466768858140854f8bc73...
```

---

### ERC20 Transfer

**Source:** [examples/erc20-transfer/main.rs](examples/erc20-transfer/main.rs)

```bash
cd examples/erc20-transfer && cargo run
```

```
Shadow-EVM: ERC20 Transfer

Sender:    0x0101010101010101010101010101010101010101
Token:     0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
Recipient: 0x0202020202020202020202020202020202020202
Amount:    1000 tokens

--- Result ---
Status:     Reverted
Gas used:   100000
Commitment: 0x18c36b29a50acc298b35698cb05be672...

Note: Minimal bytecode returns immediately.
Deploy real ERC20 bytecode for full functionality.
```

---

### DEX Swap

**Source:** [examples/uniswap-swap/main.rs](examples/uniswap-swap/main.rs)

```bash
cd examples/uniswap-swap && cargo run
```

```
Shadow-EVM: DEX Swap

Trader: 0x0101010101010101010101010101010101010101
Router: 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
Swap:   0.5 WETH -> USDC (min 1000)

--- Result ---
Gas used:        300000
Pre-state root:  0x8ae32ef342531ec44bd2b5c2b42861...
Post-state root: 0x345dc80b94ed3ba59fc390cd5c8a99...
Commitment:      0xb6b50cf8f79184821fcc3bd9cfcaf5...

--- Use Case ---
1. Run complex routing off-chain
2. Generate ZK proof of optimal path
3. Submit proof on-chain (~230k gas)
4. Avoid paying gas for failed routes
```

---

## Smart Contract Integration

### Deploy the Verifier

```solidity
ShadowEVMVerifier verifier = new ShadowEVMVerifier(
    RISC_ZERO_VERIFIER_ADDRESS,
    IMAGE_ID
);
```

### Verify a Proof On-Chain

```solidity
ShadowEVMVerifier.ExecutionCommitment memory commitment = verifier.verify(seal, journal);

bytes32 preState = commitment.preStateRoot;
bytes32 postState = commitment.postStateRoot;
```

---

## Performance

| Metric | Value |
|--------|-------|
| Core Tests | 38 passing |
| Contract Tests | 9 passing |
| Proof Size (Groth16) | ~256 bytes |
| On-Chain Verification | ~230k gas |

---

## Security

- **Computational Integrity**: STARK proofs guarantee correct EVM execution
- **State Binding**: Cryptographic commitments link input to output
- **Deterministic**: Same inputs always produce identical outputs

---

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture](docs/architecture.md) | System design |
| [Proof System](docs/proof-system.md) | Cryptographic details |
| [ZK Design](docs/zk-design.md) | ZK constraints |
| [Roadmap](docs/roadmap.md) | Milestones |

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

GPL-3.0 - See [LICENSE](LICENSE)
