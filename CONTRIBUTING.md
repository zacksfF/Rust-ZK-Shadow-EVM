# Contributing to Shadow-EVM

Thank you for your interest in contributing to Shadow-EVM! This document provides guidelines and instructions for contributing.

## 🚀 Getting Started

### Prerequisites

- Rust 1.75+
- RISC Zero toolchain (`curl -L https://risczero.com/install | bash && rzup install`)
- Foundry (`curl -L https://foundry.paradigm.xyz | bash && foundryup`)

### Setup

```bash
git clone https://github.com/zacksfF/Rust-ZK-Shadow-EVM.git
cd Rust-ZK-Shadow-EVM
cargo build --workspace
cargo test --workspace
```

## 📋 Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
```

### 2. Make Changes

- Follow the existing code style
- Add tests for new functionality
- Update documentation as needed

### 3. Test Your Changes

```bash
# Rust tests
cargo test --workspace

# Solidity tests
cd contracts && forge test

# Clippy lints
cargo clippy --workspace -- -D warnings

# Format check
cargo fmt --check
```

### 4. Submit a Pull Request

- Write a clear PR description
- Reference any related issues
- Ensure CI passes

## 🏗️ Project Structure

| Crate | Purpose |
|-------|---------|
| `shadow-evm-core` | Core EVM execution library |
| `shadow-evm-guest` | RISC Zero guest program |
| `shadow-evm-host` | Prover CLI application |

## 📝 Code Style

### Rust

- Use `rustfmt` for formatting
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Add doc comments for public items
- Use `thiserror` for error types

### Solidity

- Use `forge fmt` for formatting
- Follow Solidity style guide
- Add NatSpec comments for public functions

## 🧪 Testing Guidelines

### Unit Tests

- Place tests in the same file using `#[cfg(test)]`
- Test both success and error cases
- Use descriptive test names

### Integration Tests

- Place in `tests/` directory
- Test full execution flows
- Mock external dependencies

## 📚 Documentation

- Update README.md for user-facing changes
- Update docs/ for architectural changes
- Add inline comments for complex logic

## 🐛 Bug Reports

Please include:
1. Steps to reproduce
2. Expected behavior
3. Actual behavior
4. Environment details (OS, Rust version, etc.)

## 💡 Feature Requests

Please include:
1. Use case description
2. Proposed solution
3. Alternatives considered

## 📜 License

By contributing, you agree that your contributions will be licensed under GPL-3.0.

## 🙏 Thank You!

Your contributions help make Shadow-EVM better for everyone!
