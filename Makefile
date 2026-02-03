# Shadow-EVM Makefile
.PHONY: all build test prove verify clean fmt lint contracts

# Default target
all: build test

# Build everything
build:
	@echo "Building Shadow-EVM..."
	cargo build --release
	@echo "Rust build complete"

# Build guest only
build-guest:
	@./scripts/build_guest.sh

# Run all tests
test: test-rust test-contracts

# Rust tests
test-rust:
	@echo "Running Rust tests..."
	cargo test -p shadow-evm-core
	@echo "Rust tests passed"

# Contract tests
test-contracts:
	@echo "Running Solidity tests..."
	cd contracts && FOUNDRY_OFFLINE=true forge test -vvv
	@echo "Solidity tests passed"

# Format check
fmt:
	cargo fmt --all -- --check

# Lint check (core only, host requires RISC Zero setup)
lint:
	cargo clippy -p shadow-evm-core --all-features -- -D warnings

# Lint including host (requires RISC Zero toolchain)
lint-host:
	cargo clippy --all-targets --all-features -- -D warnings

# Generate proof (usage: make prove INPUT=input.json)
prove:
ifndef INPUT
	$(error INPUT is required. Usage: make prove INPUT=input.json)
endif
	@./scripts/prove.sh $(INPUT) $(or $(OUTPUT),proof.bin)

# Verify proof (usage: make verify PROOF=proof.bin)
verify:
ifndef PROOF
	$(error PROOF is required. Usage: make verify PROOF=proof.bin)
endif
	@./scripts/verify.sh $(PROOF)

# Build contracts
contracts:
	@echo "Building Solidity contracts..."
	cd contracts && forge build
	@echo "Contract build complete"

# Clean build artifacts
clean:
	cargo clean
	cd contracts && forge clean
	@echo "Cleaned"

# Development mode proof (fast, no real ZK)
prove-dev:
ifndef INPUT
	$(error INPUT is required. Usage: make prove-dev INPUT=input.json)
endif
	RISC0_DEV_MODE=1 ./scripts/prove.sh $(INPUT) $(or $(OUTPUT),proof.bin)

# CI target
ci: fmt lint test
