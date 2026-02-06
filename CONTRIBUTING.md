# Contributing to Norn Protocol

Thank you for your interest in contributing to the Norn Protocol. This document provides guidelines and information to help you get started.

## Getting Started

1. **Fork** the repository on GitHub.
2. **Clone** your fork locally:
   ```bash
   git clone https://github.com/<your-username>/norn-protocol.git
   cd norn-protocol
   ```
3. **Create a branch** for your work:
   ```bash
   git checkout -b feat/your-feature-name
   ```
4. Make your changes, commit, and push to your fork.
5. Open a **Pull Request** against the `main` branch.

## Development Setup

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)

The repository includes a `rust-toolchain.toml` that pins the stable channel with `clippy` and `rustfmt` components.

### Build

```bash
cargo build --workspace
```

### Test

```bash
cargo test --workspace
```

### Lint

```bash
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

All three commands must pass before submitting a pull request.

## Pull Request Process

### Branch Naming

Use descriptive branch names with a type prefix:

- `feat/description` -- New feature
- `fix/description` -- Bug fix
- `refactor/description` -- Code refactoring
- `docs/description` -- Documentation changes
- `test/description` -- Test additions or fixes

### Commit Style

This project follows [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/):

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Common types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`.

Examples:

- `feat: add batch transfer support to Thread engine`
- `fix: resolve overflow in fee calculation`
- `test: add regression tests for Merkle proof verification`

### PR Description

Include in your pull request description:

- **What** the change does
- **Why** the change is needed
- **How** to test it
- Any **breaking changes** or migration notes

## Code Style

- Run `cargo fmt` before committing. The project uses default `rustfmt` settings.
- Run `cargo clippy --workspace -- -D warnings` and resolve all warnings.
- Do not use `unsafe` code unless strictly necessary and clearly justified in comments.
- Use `thiserror` for error enums. Follow the existing error patterns in each crate.
- Use `borsh` for serialization of protocol types.

## Testing

- All new code must include tests.
- `cargo test --workspace` must pass with no failures.
- Add regression tests for any bug fix.
- End-to-end tests go in the relevant crate's `tests/` directory.

## Reporting Issues

Use [GitHub Issues](https://github.com/augmnt/norn-protocol/issues) to report bugs or request features. When reporting a bug, include:

- Steps to reproduce
- Expected behavior
- Actual behavior
- Rust version (`rustc --version`)
- Operating system

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.
