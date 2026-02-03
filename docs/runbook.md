# Shadow-EVM Runbook

Step-by-step guide for generating and verifying ZK proofs.

## Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install RISC Zero
curl -L https://risczero.com/install | bash
rzup install

# Install Foundry
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

## Quick Commands

| Task | Command |
|------|---------|
| Build all | `make build` |
| Run tests | `make test` |
| Generate proof | `make prove INPUT=input.json` |
| Verify proof | `make verify PROOF=proof.bin` |

---

## 1. Build the Project

```bash
# Build core library + host
cargo build --release

# Build Solidity contracts
cd contracts && forge build
```

## 2. Create Execution Input

Create `input.json`:

```json
{
  "block_env": {
    "number": 1,
    "timestamp": 1700000000,
    "gas_limit": 30000000,
    "coinbase": "0x0000000000000000000000000000000000000000",
    "base_fee": "0x0",
    "chain_id": 1
  },
  "tx": {
    "caller": "0x1234567890123456789012345678901234567890",
    "to": "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd",
    "value": "0x1000",
    "data": "0x",
    "gas_limit": 21000,
    "gas_price": "0x1"
  },
  "pre_state": {}
}
```

## 3. Generate a Proof

```bash
# Development mode (fast, no real proof)
RISC0_DEV_MODE=1 cargo run --release -p shadow-evm-host -- prove --input input.json --output proof.bin

# Production mode (slow, real Groth16 proof)
cargo run --release -p shadow-evm-host -- prove --input input.json --output proof.bin
```

## 4. Verify Locally

```bash
cargo run --release -p shadow-evm-host -- verify --proof proof.bin
```

## 5. Deploy Verifier Contract

```bash
cd contracts

# Deploy to local network
forge script script/Deploy.s.sol --broadcast --rpc-url http://localhost:8545

# Deploy to Sepolia
forge script script/Deploy.s.sol --broadcast --rpc-url $SEPOLIA_RPC --private-key $PRIVATE_KEY --verify
```

## 6. Submit Proof On-Chain

```bash
# Export proof for Solidity
cargo run --release -p shadow-evm-host -- export --proof proof.bin --output calldata.json

# Submit via cast
cast send $VERIFIER_ADDRESS "verify(bytes,bytes32)" $(cat calldata.json) --rpc-url $RPC_URL --private-key $PRIVATE_KEY
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `risc0-build` panic | Ensure `[package.metadata.risc0]` in host Cargo.toml |
| Slow proof generation | Use `RISC0_DEV_MODE=1` for testing |
| Foundry proxy crash | Set `FOUNDRY_OFFLINE=true` |
| Memory limit exceeded | Reduce state size in input |
