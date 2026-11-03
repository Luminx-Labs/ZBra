# Repository Guidelines

## Project Structure & Module Organization

ZBra is a chess engine whose essential libraries & NNUE interface is written in Rust and NNUE is trained through Python.

- The Rust package `xtask` is made for workspace automation; Its binary's source code is located in `utils/xtask`.
- `.xtask.toml` is the repository-level source of truth for Python, PyTorch, ABI, model, and utility tools with justification of their presence.
- `docs/` contains the mdBook documentation (`docs/src/`) as **Development Guide** and also contains **API Guidance & ADRs**.
- `README.md` provides the project overview.
- The Workspace is organized by folders:
    - `utils` for utility tools.
    - `nnue` for Python Codebase to train the NNUE.
    - `crates` for Rust crates

> Note: The UCI(Unified Chess Interface) of engine is written in Rust.

## Build, Test, and Development Commands

Run commands from the repository root:

```sh
cargo fmt --all                 # Format Rust sources
cargo check --workspace         # Type-check the workspace
cargo test --workspace          # Run Rust tests (as they are added)
cargo clippy --workspace --all-targets --all-features
cargo xtask doctor              # Inspect configured tools and metadata
cargo xtask doctor --install    # Install missing Python tooling and sync its environment
cargo xtask abi                 # Validate Python ABI and configured PyTorch
cargo xtask check-workspace     # Run the xtask Rust workspace check
cargo xtask hooks               # Install the repository pre-commit hook
cargo xtask utils               # List configured Rust/C utility binaries
cargo xtask build-utils         # Build configured utility binaries
cargo xtask pre-commit          # Run formatting, Clippy, and ABI checks
cargo xtask changelog           # Generate CHANGELOG.md from Git history
cargo xtask version             # Show the workspace version
cargo xtask build               # Build release binaries and utilities
cargo xtask dist                # Create a ZIP distribution and SHA-256 file
cargo xtask release             # Check, prepare, commit, and tag a release
cargo xtask ci                  # Run the CI check sequence
cargo run -p docs_preprocessor  # Run the Docs docs_preprocessor on the Documentation Book
```

Use the pinned toolchain in `rust-toolchain.toml`. Keep Python environment details synchronized with `.xtask.toml`; do not hard-code machine-specific interpreter paths.

## Coding Style & Naming Conventions

Use Rust 2024 edition formatting with four-space indentation and run `cargo fmt --all` before committing. Follow Rust naming conventions: `snake_case` for functions/modules, `UpperCamelCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Respect the workspace lints.

## Testing Guidelines

Add unit tests beside the module under test and integration tests under a crate’s `tests/` directory when applicable. Prefer deterministic tests for configuration parsing, command selection, ABI detection, and model metadata. Run `cargo nextest --workspace` and the relevant `cargo xtask` command before submitting changes.

## Commit & Pull Request Guidelines

Use short Conventional Commit subjects with a focused scope, for example `feat(xtask): validate model metadata` or `fix(config): reject unsupported ABI`. Pull requests should explain the behavior change, list validation commands and results, identify configuration/model format changes, and link the relevant issue. Include sample CLI output when changing developer tooling; include screenshots only for documentation presentation changes.

## Configuration and Model Safety

Treat `.xtask.toml` as shared project configuration. Do not commit credentials, local virtual environments, generated models, or unchecked-in training data. When changing a model format or ABI expectation, update its tracked metadata and document compatibility implications.

# Roadmap for Development of ZBra

## 1. Phase - I: Foundation and Basic Tooling

In this phase, it is expected to build a strong, solid and reliable foundation as to ensure the smooth development of project. Here are the following main task for **Phase-I**:

- **Chess** library: a chess library `zbra_chess` should be written and tested properly.
- **Python** bindings: crate `zbra_py` should be written and checked thoroughly.
- _NNUE_ interface: **NNUE** interface should be written in Rust crate `zbra_nnue` and tested.
- Game Parser: a _pgn_ format parser should be written in crate `zbra_parser`

## 2. Phase - II: Python Interface & Algorithms

In this phase, the **Python** codebase should be made for training of NNUE, with some efficient tools to work on later. Algorithms like **Alpha-Beta**, **MINI MAX**, etc should be added in crate `zbra_math`

## 3. Phase - III: UCI and NNUE Training

In this phase, UCI should be done and well tested and NNUE should be trained for basic openings and some **ELITE** Games. Also, the NNUE interface should be customized & written in C or Rust according to the pace of training if required. 

## 3. Phase - IV: Utilities for Training Escalation & Optimization for System Usage

In order to make engine **performant**, utilities should be made for increasing the pace of training & tracking the **_ELO_** in any circumstance. For ensuring the maximum **Efficiency**, we must ensure reducing the system usage.

## 4. Phase - V: Distillation and Self-Play Management

In this phase, the **Distillation** of engine should done with appropriate NNUE based strong and powerful engine.
