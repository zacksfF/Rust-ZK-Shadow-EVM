# Shadow-EVM ZK Design

This document covers the zero-knowledge specific design decisions and constraints.

## Design Goals

1. **ZK Compatibility**: All code must run inside a ZK-VM
2. **Determinism**: Identical inputs must produce identical outputs
3. **Efficiency**: Minimize cycle count for faster proofs
4. **Auditability**: Clear separation of concerns

## no_std Compatibility

The core library is designed to work without the standard library:

```rust
#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
```

### Why no_std?

ZK-VMs like RISC Zero run in constrained environments without:
- System calls
- File I/O
- Network access
- Thread spawning
- Random number generation

All computation must be pure and deterministic.

### Implications

| Feature | Standard Rust | no_std for ZK |
|---------|---------------|---------------|
| Collections | `std::collections` | `alloc::collections` |
| Strings | `String` | `alloc::string::String` |
| Errors | `std::error::Error` | Custom error types |
| Hash Maps | `HashMap` | `BTreeMap` (deterministic) |
| Randomness | `rand` |  Not allowed |
| Time | `std::time` | Provided via input |

## Determinism Requirements

### State Ordering

The `InMemoryDB` uses `BTreeMap` instead of `HashMap`:

```rust
pub struct InMemoryDB {
    // BTreeMap provides deterministic iteration order
    pub accounts: BTreeMap<Address, AccountState>,
}
```

This ensures:
- Same accounts always hash to same state root
- Iteration order is consistent across runs
- No randomness from hash map bucket ordering

### Serialization

Bincode serialization is used for deterministic encoding:

```rust
fn hash_struct<T: Serialize>(value: &T) -> Hash {
    let bytes = bincode::serialize(value).unwrap();
    keccak256(&bytes)
}
```

### Floating Point

Floating point operations are NOT used because:
- Different hardware may produce different results
- Rounding can vary between implementations
- EVM uses fixed-point arithmetic (U256)

## Cycle Optimization

Every CPU instruction costs cycles in the ZK-VM. Optimizations include:

### 1. Lazy Hashing

Only compute hashes when needed:

```rust
impl ExecutionOutput {
    // Hash computed on demand, not stored
    pub fn hash(&self) -> Hash {
        hash_struct(self)
    }
}
```

### 2. Minimal Memory Allocation

Reuse buffers where possible:

```rust
// Bad: Allocates new vec each time
fn process(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}

// Good: Minimize allocations
fn process(data: &[u8], out: &mut Vec<u8>) {
    out.clear();
    out.extend_from_slice(data);
}
```

### 3. Avoid Complex Algorithms

Simple algorithms often perform better in ZK:
- Linear search over small sets vs hash lookup
- Iterative over recursive solutions
- Fixed-size buffers over dynamic allocation

## State Management

### Pre-State Verification

The guest trusts the pre-state provided by the host, but commits to it:

```rust
let pre_state_root = input.pre_state.compute_state_root();
// This becomes part of the commitment
```

The on-chain verifier can check this pre-state root against known chain state.

### Post-State Commitment

After execution, the post-state is committed:

```rust
let post_state_root = output.post_state.compute_state_root();
// Included in ExecutionCommitment
```

This creates a verifiable chain: `pre_state → execution → post_state`

## Transaction Execution

### Block Context

Block-level parameters are provided as input:

```rust
pub struct BlockEnv {
    pub number: u64,
    pub timestamp: u64,
    pub gas_limit: u64,
    pub coinbase: Address,
    pub base_fee: U256,
    pub prev_randao: Hash,
    pub chain_id: u64,
}
```

All values must be provided—no syscalls to query chain state.

### Gas Accounting

Gas is tracked but not enforced at the network level:

```rust
pub struct ExecutionOutput {
    pub gas_used: u64,
    pub gas_refunded: u64,
    // ...
}
```

The proof shows how much gas was used, but doesn't prevent execution.

## Error Handling

Errors in the guest abort proof generation:

```rust
fn main() {
    let result = ZkExecutor::execute(input);
    
    match result {
        Ok((output, commitment)) => {
            commit_output(&commitment);
        }
        Err(_) => {
            // Panic aborts the ZK-VM
            panic!("EVM execution failed");
        }
    }
}
```

The host should validate inputs before proving to avoid wasted cycles.

## Memory Layout

The guest has limited memory available:

| Region | Size | Purpose |
|--------|------|---------|
| Stack | 1 MB | Local variables, call frames |
| Heap | ~256 MB | Dynamic allocations |
| Code | ~16 MB | Guest program ELF |

Large state sets may exceed memory limits.

## Security Considerations

### Input Validation

The host must validate:
1. Pre-state is authentic (matches on-chain)
2. Transaction is well-formed
3. Block parameters are correct

### Trusted Setup

Groth16 requires a trusted setup:
- RISC Zero provides the setup
- Parameters are publicly verifiable
- Compromise would affect all proofs

### Side Channels

ZK proofs can leak information through:
- Cycle count (execution time)
- Memory access patterns

Shadow-EVM doesn't currently mitigate these—consider if privacy is required.

## Testing Strategy

### Unit Tests

Core library tests run in standard environment:

```bash
cargo test -p shadow-evm-core
```

### Integration Tests

Full ZK execution tests:

```bash
# Uses dev mode for speed
cargo test -p shadow-evm-host --features test
```

### Cycle Counting

Profile cycle usage:

```bash
RISC0_DEV_MODE=0 cargo run --release -- execute --input test.json --verbose
```

## Future Optimizations

1. **Precompiles**: Accelerated crypto operations
2. **Memory Pooling**: Reduce allocation overhead
3. **Parallel Proving**: Split large computations
4. **Custom opcodes**: Skip expensive operations when safe
