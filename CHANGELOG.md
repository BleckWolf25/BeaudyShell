# Change Log

All notable changes to the **BeaudyShell** project will be documented in this file.
Format follows [Keep a Changelog](http://keepachangelog.com/), versioning follows
[Semantic Versioning](https://semver.org/).

---

## [Unreleased]

## [0.2.0] - 2026-07-23

### Added
- **Signal & Exit Handling**:
  - `Ctrl+C` clears current input prompt instead of terminating shell process.
  - `Ctrl+D` on an empty prompt cleanly exits BeaudyShell.
- **Persistent History**: Command history automatically saved to and loaded from `~/.beaudy_history`.
- **Subdirectory Path Autocompletion**: Tab completion now searches subdirectories, parent paths (`..`), and home shorthand (`~`).
- **Pipeline & Redirection Support**: Builtin and external commands support piping (`|`), overwriting (`>`), and appending (`>>`).
- **Environment Variable Management & Expansion**:
  - Added `export` builtin to list and set environment variables.
  - Automatic `$VAR` and `${VAR}` variable expansion before command execution.
- **Command Aliasing**:
  - Added `alias` and `unalias` builtins.
  - Saved aliases persist automatically in `~/.beaudy.toml`.
- **Reverse History Search**: `Ctrl+R` interactive reverse-i-search mode through command history.
- **Configurable Default Subshell**: `default_shell` setting in `~/.beaudy.toml` (e.g. `pwsh`, `powershell.exe`, `bash`, `zsh`, `sh`).
- **Built-in Tool Enhancements**:
  - `bls`: Added `-a`/`--all` (hidden dotfiles), `-R`/`--recursive` (directory tree view), and colorized output (Cyan for dirs, Green for executables, Magenta for symlinks).
  - `btrash`: Added `list`/`--list`, `restore <file>`/`--restore <file>`, and `empty`/`--empty` subcommands with cross-filesystem fallback handling (`EXDEV`).
  - `bmemo`: Formatted timestamps to `[YYYY-MM-DD HH:MM]` and added `bmemo list` subcommand.
  - Directory Stack: Added `pushd`, `popd`, and `dirs` builtins backed by thread-safe `DIR_STACK`.
- **Prompt UX & Editing Shortcuts**:
  - Active Git branch and dirty state indicator in prompt (`[main*]`).
  - Command exit status indicator: `➜` in green on success (`0`) or red `➜ [code]` on error.
  - Terminal shortcuts: `Ctrl+L` (clear screen), `Ctrl+W` (delete previous word), `Ctrl+U` (clear line before cursor), `Ctrl+K` (clear line after cursor), `Alt+B` / `Ctrl+Left` (jump back one word), `Alt+F` / `Ctrl+Right` (jump forward one word).
- **Documentation & Unit Tests**:
  - 78 unit tests covering REPL, autocomplete, history, prompt, config, builtins, and router logic.
  - 14 executable documentation tests (`Doc-tests`) across `beaudy-a11y`, `beaudy-builtins`, and `beaudy-router`. Total: 92 tests passing.
- **Build Automation**: Added `make dev` target to `Makefile`.

## [0.1.1], 2026-07-16

### Fixed
- Fixed workflow permissions and action versions in CI and Release configurations.

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
