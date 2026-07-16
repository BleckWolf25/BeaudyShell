#!/usr/bin/env bash
# ============================================================================
# install.sh, BeaudyShell installer for Unix/Linux/macOS
# ============================================================================
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/BleckWolf25/BeaudyShell/main/install.sh | bash
#   Or run locally: bash install.sh [--prefix /usr/local]
#
# Options:
#   --prefix PATH   Install location (default: /usr/local)
#   --version TAG   Version to install (default: latest)
#   --help          Show this help message
# ============================================================================
set -euo pipefail

REPO="BleckWolf25/BeaudyShell"
BINARY_NAME="beaudy-entry"
INSTALL_NAME="beaudy"
DEFAULT_PREFIX="/usr/local"
PREFIX="${DEFAULT_PREFIX}"
VERSION="latest"

BOLD="\033[1m"
GREEN="\033[32m"
YELLOW="\033[33m"
RED="\033[31m"
RESET="\033[0m"

info()  { echo -e "${BOLD}${GREEN}[info]${RESET}  $*"; }
warn()  { echo -e "${BOLD}${YELLOW}[warn]${RESET}  $*"; }
error() { echo -e "${BOLD}${RED}[error]${RESET} $*" >&2; exit 1; }

# -----------------------------------------------------------------------
# Parse arguments
# -----------------------------------------------------------------------
while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)
      PREFIX="$2"; shift 2 ;;
    --version)
      VERSION="$2"; shift 2 ;;
    --help|-h)
      echo "Usage: install.sh [--prefix PATH] [--version TAG]"
      exit 0 ;;
    *)
      error "Unknown argument: $1" ;;
  esac
done

BIN_DIR="${PREFIX}/bin"

# -----------------------------------------------------------------------
# Detect OS and architecture
# -----------------------------------------------------------------------
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
  Linux)  OS_SLUG="linux" ;;
  Darwin) OS_SLUG="macos" ;;
  *)      error "Unsupported OS: ${OS}" ;;
esac

case "${ARCH}" in
  x86_64)  ARCH_SLUG="x86_64" ;;
  aarch64|arm64) ARCH_SLUG="aarch64" ;;
  *)        error "Unsupported architecture: ${ARCH}" ;;
esac

ASSET_NAME="beaudy-${OS_SLUG}-${ARCH_SLUG}"

# -----------------------------------------------------------------------
# Resolve version
# -----------------------------------------------------------------------
if [[ "${VERSION}" == "latest" ]]; then
  info "Fetching latest release tag…"
  if command -v curl &>/dev/null; then
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
      | grep '"tag_name"' | head -1 | cut -d'"' -f4)
  elif command -v wget &>/dev/null; then
    VERSION=$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" \
      | grep '"tag_name"' | head -1 | cut -d'"' -f4)
  else
    error "curl or wget is required to fetch releases."
  fi
  [[ -n "${VERSION}" ]] || error "Could not determine the latest version."
fi

DOWNLOAD_BASE="https://github.com/${REPO}/releases/download/${VERSION}"
BINARY_URL="${DOWNLOAD_BASE}/${ASSET_NAME}"
CHECKSUM_URL="${DOWNLOAD_BASE}/${ASSET_NAME}.sha256"

info "Installing BeaudyShell ${VERSION} (${OS_SLUG}/${ARCH_SLUG})…"

# -----------------------------------------------------------------------
# Download binary
# -----------------------------------------------------------------------
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

TMP_BIN="${TMP_DIR}/${BINARY_NAME}"
TMP_SHA="${TMP_DIR}/${BINARY_NAME}.sha256"

info "Downloading ${ASSET_NAME}…"
if command -v curl &>/dev/null; then
  curl -fsSL "${BINARY_URL}" -o "${TMP_BIN}"
  curl -fsSL "${CHECKSUM_URL}" -o "${TMP_SHA}"
else
  wget -qO "${TMP_BIN}" "${BINARY_URL}"
  wget -qO "${TMP_SHA}" "${CHECKSUM_URL}"
fi

# -----------------------------------------------------------------------
# Verify checksum
# -----------------------------------------------------------------------
info "Verifying checksum…"
EXPECTED=$(awk '{print $1}' "${TMP_SHA}")
if command -v sha256sum &>/dev/null; then
  ACTUAL=$(sha256sum "${TMP_BIN}" | awk '{print $1}')
elif command -v shasum &>/dev/null; then
  ACTUAL=$(shasum -a 256 "${TMP_BIN}" | awk '{print $1}')
else
  warn "No sha256 tool found, skipping checksum verification."
  ACTUAL="${EXPECTED}"
fi

if [[ "${EXPECTED}" != "${ACTUAL}" ]]; then
  error "Checksum mismatch!\n  Expected: ${EXPECTED}\n  Actual:   ${ACTUAL}"
fi
info "Checksum OK."

# -----------------------------------------------------------------------
# Install binary
# -----------------------------------------------------------------------
chmod +x "${TMP_BIN}"

if [[ ! -d "${BIN_DIR}" ]]; then
  warn "${BIN_DIR} does not exist, creating it…"
  mkdir -p "${BIN_DIR}"
fi

DEST="${BIN_DIR}/${INSTALL_NAME}"

if [[ -w "${BIN_DIR}" ]]; then
  mv "${TMP_BIN}" "${DEST}"
else
  info "Elevated permissions required to install to ${BIN_DIR}…"
  sudo mv "${TMP_BIN}" "${DEST}"
fi

# -----------------------------------------------------------------------
# Verify installation
# -----------------------------------------------------------------------
if command -v beaudy &>/dev/null; then
  info "${BOLD}BeaudyShell ${VERSION} installed successfully!${RESET}"
  info "Run '${BOLD}beaudy${RESET}' to start the shell."
else
  warn "Binary installed to ${DEST}, but it may not be on your PATH."
  warn "Add '${BIN_DIR}' to your PATH, or run it directly: ${DEST}"
fi
