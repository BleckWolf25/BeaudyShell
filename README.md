# BeaudyShell

> A modern, extensible, and accessible command‑line shell written in Rust.

BeaudyShell provides a rich, interactive REPL experience with syntax highlighting, inline autocomplete, command history, and built‑in utilities such as `cd`, `bhelp`, and `bls`. It also supports OSC 133 accessibility markers for screen‑reader navigation.

## 🚀 Getting Started

### Prerequisites

- **Rust** stable (install via `rustup`)
- **make** (for the provided Makefile targets)

### Installation

```bash
# Clone the repository
git clone https://github.com/BleckWolf25/BeaudyShell.git
cd BeaudyShell

# Build the project (debug)
make build

# Run the shell
./target/debug/beaudy-shell
```

For an optimized binary:

```bash
make release   # builds with LTO, strip, and opt‑level 3
./target/release/beaudy-shell
```

### Quick Start Commands

- `bhelp` – Show help for built‑in commands and shortcuts.
- `bls` – List directory contents with type, size, and modification time.
- `cd <dir>` – Change directory (supports `~` and `-`).

## 🛠️ Development

The project is a multi‑crate workspace:

- `beaudy-entry` – REPL loop, input handling, syntax highlighting.
- `beaudy-router` – PTY management, `cd` state, command routing.
- `beaudy-builtins` – Built‑in commands implementation.
- `beaudy-a11y` – Accessibility helpers (OSC 133 markers).

### Common Makefile Targets

| Target | Description |
|--------|-------------|
| `setup` | Install development dependencies (if any). |
| `build` | Compile all crates in debug mode. |
| `release` | Compile with release profile (optimised). |
| `run` | Build and execute the shell. |
| `test` | Run all unit tests (`cargo test --workspace`). |
| `fmt` | Run `cargo fmt --all`. |
| `lint` | Run `cargo clippy --workspace --all-targets --all-features -D warnings`. |
| `package` | Build release binary, generate SHA‑256 checksum, and create installer script. |

## 📄 Documentation

- **Configuration** – The shell reads `~/.beaudy.toml`. See the file for available options (`prompt_style`, `prompt_color`).
- **Accessibility** – OSC 133 markers (`A`, `B`, `C`, `D`) enable screen‑reader navigation.
- **Built‑ins** – `bhelp`, `bls`, and `cd` are implemented in `beaudy-builtins` and `beaudy-router`.

## 🤝 Contributing

Please read our [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on reporting bugs, suggesting enhancements, and submitting pull requests. By contributing you agree to follow our [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## 📄 License

Distributed under the MIT License. See `LICENSE` for details.

## 🔒 Security

If you discover a security vulnerability, please see our [SECURITY.md](SECURITY.md) for reporting instructions.

---

Built with ❤️ using Rust, crossterm, and portable‑pty.
