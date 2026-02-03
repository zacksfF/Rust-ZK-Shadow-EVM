#!/bin/bash
# Build the Shadow-EVM guest program
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "Building Shadow-EVM Guest..."

# Check for RISC Zero toolchain
if ! command -v cargo-risczero &> /dev/null; then
    echo "ERROR: RISC Zero toolchain not found. Install with:"
    echo "  curl -L https://risczero.com/install | bash && rzup install"
    exit 1
fi

cd "$PROJECT_ROOT"

# Build the guest (via host build script)
cargo build --release -p shadow-evm-host

echo "Guest build complete!"
echo "Image ID available in target/release/build/shadow-evm-host-*/out/methods.rs"
