#!/bin/bash
# Shadow-EVM Setup Script
# Run this script to set up the complete development environment

set -e

echo " Shadow-EVM Setup Script"
echo "=========================="
echo ""

# Check Rust
echo "Checking Rust..."
if ! command -v cargo &> /dev/null; then
    echo " Rust not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
else
    echo " Rust found: $(rustc --version)"
fi

# Check Foundry
echo ""
echo "Checking Foundry..."
if ! command -v forge &> /dev/null; then
    echo " Foundry not found. Installing..."
    curl -L https://foundry.paradigm.xyz | bash
    source $HOME/.bashrc 2>/dev/null || source $HOME/.zshrc 2>/dev/null || true
    foundryup
else
    echo " Foundry found: $(forge --version)"
fi

# Check RISC Zero
echo ""
echo "Checking RISC Zero..."
if ! command -v cargo-risczero &> /dev/null; then
    echo " RISC Zero not found."
    echo "To install RISC Zero, run:"
    echo "  curl -L https://risczero.com/install | bash"
    echo "  rzup install"
    echo ""
    echo "Skipping RISC Zero for now (host build will fail without it)"
else
    echo " RISC Zero found"
fi

# Build core library
echo ""
echo "Building core library..."
cargo build -p shadow-evm-core

# Run core tests
echo ""
echo "Running core tests..."
cargo test -p shadow-evm-core

# Build and test Solidity contracts
echo ""
echo "Building Solidity contracts..."
cd contracts
forge build

echo ""
echo "Running Solidity tests..."
forge test
cd ..

echo ""
echo "================================"
echo " Setup complete!"
echo ""
echo "Next steps:"
echo "  1. Install RISC Zero if not already installed:"
echo "     curl -L https://risczero.com/install | bash && rzup install"
echo ""
echo "  2. Build the host CLI:"
echo "     cargo build -p shadow-evm-host --release"
echo ""
echo "  3. Run examples:"
echo "     cd examples/simple-call && cargo run"
echo ""
echo "  4. Generate a proof:"
echo "     ./target/release/shadow-evm sample --output tx.json"
echo "     ./target/release/shadow-evm prove --input tx.json --output proof.bin --dev"
echo ""
