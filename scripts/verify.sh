#!/bin/bash
# Verify a Shadow-EVM ZK proof locally
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

PROOF="${1:-}"

usage() {
    echo "Usage: $0 <proof.bin>"
    echo ""
    echo "Example:"
    echo "  $0 proof.bin"
    exit 1
}

if [[ -z "$PROOF" ]]; then
    usage
fi

if [[ ! -f "$PROOF" ]]; then
    echo "ERROR: Proof file not found: $PROOF"
    exit 1
fi

echo "Verifying Shadow-EVM ZK Proof..."
echo "  Proof: $PROOF"

cd "$PROJECT_ROOT"

cargo run --release -p shadow-evm-host -- verify --proof "$PROOF"

echo "Proof verification complete!"
