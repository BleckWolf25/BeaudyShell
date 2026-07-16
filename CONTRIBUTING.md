# Contributing to BeaudyShell

First off, thank you for taking the time to contribute! Contributions from the community help make BeaudyShell more comprehensive, accurate, and helpful for everyone.

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

---

## Table of Contents

- [How Can I Contribute?](#how-can-i-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Enhancements](#suggesting-enhancements)
  - [Pull Requests](#pull-requests)
- [Development Setup](#development-setup)
  - [Prerequisites](#prerequisites)
  - [Setting Up Your Workspace](#setting-up-your-workspace)
  - [Development Commands](#development-commands)
- [Style & Code Guidelines](#style--code-guidelines)
  - [Rust Coding Style](#rust-coding-style)
  - [Project Structure Guidelines](#project-structure-guidelines)
  - [Commit Messages](#commit-messages)
- [Testing](#testing)
  - [Running Tests](#running-tests)
- [Security Vulnerabilities](#security-vulnerabilities)

---

## How Can I Contribute?

### Reporting Bugs

We use GitHub Issues to track bug reports. Before submitting a bug report, please:

1. Search existing issues to ensure it hasn't been reported.
2. Verify the bug on a clean checkout of the repository.
3. Provide:
   - BeaudyShell version (`beaudy-shell --version`)
   - Platform details (OS, architecture)
   - Steps to reproduce the issue
   - Relevant log output (`~/.beaudy.log` if applicable)

### Suggesting Enhancements

If you have ideas for new built‑ins, UI improvements, or performance enhancements:

1. Check existing issues for similar suggestions.
2. Open a Feature Request describing the motivation, proposed behavior, and any design considerations.

### Pull Requests

1. Fork the repository and create a branch from `main` (e.g., `feat/your-feature`).
2. Keep changes focused; avoid mixing unrelated updates.
3. Follow the project's coding style (see below).
4. Ensure `cargo fmt`, `cargo clippy`, and `cargo test` all pass.
5. Submit a PR with a clear description and reference any related issues.

---

## Development Setup

### Prerequisites

- **Rust** stable (via `rustup`)
- **cargo** (comes with Rust)
- **make** (for the provided Makefile targets)

### Setting Up Your Workspace

```bash
# Clone the repository
git clone https://github.com/BleckWolf25/BeaudyShell.git
cd BeaudyShell

# Build all crates
cargo build --workspace
```

### Development Commands

- **Compile & run**: `make run`
- **Run tests**: `make test`
- **Format**: `make fmt`
- **Lint**: `make lint`
- **Release build**: `make release`
- **Package**: `make package`

---

## Style & Code Guidelines

### Rust Coding Style

- Use `rustfmt` defaults (4‑space indentation).
- Prefer `snake_case` for functions/variables, `PascalCase` for types.
- Keep line length ≤ 100 characters.
- Document public items with `///` comments.
- Avoid `unwrap`/`expect` in library code; return proper `Result`.

### Project Structure Guidelines

- `crates/beaudy-entry`: REPL and high‑level orchestration.
- `crates/beaudy-router`: PTY handling, `cd` built‑in, command routing.
- `crates/beaudy-builtins`: Built‑in commands (`bhelp`, `bls`).
- `crates/beaudy-a11y`: Accessibility helpers (OSC 133 markers).

### Commit Messages

Use conventional prefixes:
- `feat:` new feature
- `fix:` bug fix
- `docs:` documentation changes
- `refactor:` internal refactor
- `test:` test additions/updates
- `chore:` build or tooling changes

Example:
```
feat: add configurable prompt style via ~/.beaudy.toml
```

---

## Testing

```bash
make test   # runs all unit tests across workspace
```

All tests should pass on both Linux and Windows CI runners.

---

## Security Vulnerabilities

Do not disclose security issues publicly. See our [SECURITY.md](SECURITY.md) for reporting instructions.
