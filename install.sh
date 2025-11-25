#!/usr/bin/env bash
set -e

# miam RSS Reader - One-line installer
# Usage: curl -sSL https://raw.githubusercontent.com/saravenpi/miam/master/install.sh | bash

REPO="saravenpi/miam"
INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="miam"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
check_prerequisites() {
    info "Checking prerequisites..."

    if ! command_exists cargo; then
        error "Cargo not found. Please install Rust: https://rustup.rs/"
    fi

    if ! command_exists git; then
        error "Git not found. Please install git."
    fi
}

# Build from source using cargo
build_from_source() {
    info "Building from source..."

    local tmp_dir
    tmp_dir="$(mktemp -d)"

    info "Cloning repository..."
    git clone --depth 1 "https://github.com/${REPO}.git" "${tmp_dir}/${BINARY_NAME}" || error "Failed to clone repository"

    cd "${tmp_dir}/${BINARY_NAME}"

    info "Building ${BINARY_NAME} (this may take a few minutes)..."
    cargo build --release || error "Build failed"

    mkdir -p "${INSTALL_DIR}"
    cp "target/release/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    cd -
    rm -rf "${tmp_dir}"
}

# Check if install directory is in PATH
check_path() {
    if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
        warn "${INSTALL_DIR} is not in your PATH"
        echo ""
        echo "Add the following to your shell configuration file:"
        echo ""
        echo "  export PATH=\"\$PATH:${INSTALL_DIR}\""
        echo ""
        echo "Shell config files:"
        echo "  - Bash: ~/.bashrc or ~/.bash_profile"
        echo "  - Zsh: ~/.zshrc"
        echo "  - Fish: ~/.config/fish/config.fish"
        echo ""
    fi
}

# Main installation flow
main() {
    echo ""
    echo "╔══════════════════════════════════════╗"
    echo "║   miam RSS Reader Installer          ║"
    echo "╚══════════════════════════════════════╝"
    echo ""

    check_prerequisites

    build_from_source

    success "${BINARY_NAME} installed successfully to ${INSTALL_DIR}/${BINARY_NAME}"

    check_path

    echo ""
    success "Installation complete!"
    echo ""
    echo "Get started:"
    echo "  1. Run: ${BINARY_NAME}"
    echo "  2. Press 'a' to add your first RSS feed"
    echo "  3. Press 'r' to refresh feeds"
    echo ""
    echo "For updates, run: ${BINARY_NAME} upgrade"
    echo "Documentation: https://github.com/${REPO}#readme"
    echo ""
}

main
