# @file Makefile
#
# @version 1.0.0
# @author BleckWolf25
# @license MIT
#
# @summary Build automation and release management for the BeaudyShell project.
#
# @description
# This Makefile provides commands for building, testing, linting, formatting,
# and releasing the BeaudyShell workspace, ensuring clean builds and code quality
# standards before packaging the final release artifacts.
#
# @since 16/07/2026
# @updated 16/07/2026
#
# ---------- CONFIGURATION
.PHONY: setup build release run dev test lint fmt clean package

# ---------- DEVELOPMENT COMMANDS
# Install required Rust components
setup:
	rustup component add clippy rustfmt

# Build the entire workspace in debug mode
build:
	cargo build

# Build the workspace in release mode for packaging
release:
	cargo build --release

# Run the BeaudyShell entry point
run:
	cargo run -p beaudy-entry

# Alias for run / dev workflow
dev: run

# Run all tests across all crates
test:
	cargo test --workspace

# Run strict linting
lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# Format code (modifies files to match rustfmt.toml)
fmt:
	cargo fmt --all

# Check formatting without modifying (useful for CI)
fmt-check:
	cargo fmt --all -- --check

# Clean build artifacts
clean:
	cargo clean

# Complete release workflow: clean, format, lint, build, and package
package: clean
	@echo "========== CLEANING CACHE AND ARTIFACTS =========="
	cargo clean
	@echo "========== FORMATTING ALL CODE =========="
	cargo fmt --all
	@echo "========== LINTING CODE =========="
	cargo clippy --workspace --all-targets --all-features -- -D warnings
	@echo "========== BUILDING RELEASE =========="
	cargo build --release
	@echo "========== PACKAGING COMPLETE =========="
	@echo "Release binary available at target/release/beaudy-entry"