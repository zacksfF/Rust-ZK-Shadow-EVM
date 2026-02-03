#!/bin/bash
# Generate a ZK proof for Shadow-EVM execution
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Default values
DEV_MODE="${RISC0_DEV_MODE:-0}"
INPUT="${1:-}"
OUTPUT="${2:-proof.bin}"

usage() {
    echo "Usage: $0 <input.json> [output.bin]"
    echo ""
    echo "Options (via environment):"
    echo "  RISC0_DEV_MODE=1    Fast mode (no real proof)"
    echo ""
    echo "Example:"
    echo "  $0 input.json proof.bin"
    echo "  RISC0_DEV_MODE=1 $0 input.json"
    exit 1
}

if [[ -z "$INPUT" ]]; then
    usage
fi

if [[ ! -f "$INPUT" ]]; then
    echo "ERROR: Input file not found: $INPUT"
    exit 1
fi

echo "Generating Shadow-EVM ZK Proof..."
echo "  Input:  $INPUT"
echo "  Output: $OUTPUT"
echo "  Mode:   $([ "$DEV_MODE" = "1" ] && echo "DEV (fast)" || echo "PRODUCTION")"

cd "$PROJECT_ROOT"

RISC0_DEV_MODE="$DEV_MODE" cargo run --release -p shadow-evm-host -- prove \
    --input "$INPUT" \
    --output "$OUTPUT"

echo "Proof generated: $OUTPUT"
