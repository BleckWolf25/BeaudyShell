# Change Log

All notable changes to the **BeaudyShell** project will be documented in this file.
Format follows [Keep a Changelog](http://keepachangelog.com/), versioning follows
[Semantic Versioning](https://semver.org/).

---

## [Unreleased]

## [0.1.0], 2026-07-16

### Added
- **Interactive REPL** with full raw-mode keystroke handling (crossterm).
- **Syntax highlighting**, cyan for builtins, green for known commands, red for unknown.
- **Inline autocomplete** suggestions shown in grey; accepted with `→` or `End`.
- **Tab cycling** through all matching completion candidates.
- **Command history** navigation with `↑` / `↓`; restores draft input on `↓` past end.
- **`cd` builtin** with full state tracking:
  - `cd`, `cd ~` → home directory.
  - `cd -` → previous directory (`OLDPWD`).
  - `~/prefix` path expansion.
- **`bls`**, accessible directory listing with name, type, size, and relative mtime.
- **`bhelp`**, built-in help screen listing all commands and keyboard shortcuts.
- **OSC 133** semantic shell markers (`A`/`B`/`C`/`D`) for screen-reader jump navigation.
- **Configuration file** (`~/.beaudy.toml`) with auto-generation of defaults:
  - `prompt_style`: `"path"`, `"compact"`, `"static"`.
  - `prompt_color`: `"cyan"`, `"green"`, `"blue"`, `"magenta"`, `"yellow"`, `"white"`, `"red"`.
- **Dynamic prompt** showing the current working directory, tildified for home paths.
- **PATH command caching** via `OnceLock<HashSet<String>>`, eliminates repeated disk scans on every keystroke.
- **PTY process execution** via `portable-pty` with bidirectional I/O threading; terminal size propagated to child processes so full-screen TUIs (e.g. `vim`, `htop`) render correctly.
- **Release build profile** with `opt-level = 3`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true`.
- **GitHub Actions workflows**:
  - `ci.yml`, format check, clippy, test suite, and release build on every push/PR.
  - `release.yml`, cross-platform binary release (Linux x86_64 + aarch64, Windows x86_64) with SHA-256 checksums, triggered by `v*.*.*` tags.
- **`install.sh`**, one-line Unix installer with OS/arch detection and checksum verification.
- **`Makefile`** targets: `setup`, `build`, `release`, `run`, `test`, `lint`, `fmt`, `fmt-check`, `clean`, `package`.

### Workspace crates
| Crate             | Role                                                            |
|-------------------|-----------------------------------------------------------------|
| `beaudy-entry`    | REPL entry point, input loop, syntax highlighting, autocomplete |
| `beaudy-router`   | Command routing, `cd` interception, PTY subprocess management   |
| `beaudy-builtins` | `bls` and `bhelp` implementations                               |
| `beaudy-a11y`     | OSC 133 marker generation                                       |
