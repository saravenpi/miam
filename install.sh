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

# Detect OS and architecture
detect_platform() {
    local os arch

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)
            os="linux"
            ;;
        Darwin)
            os="macos"
            ;;
        *)
            error "Unsupported operating system: $os"
            ;;
    esac

    case "$arch" in
        x86_64 | amd64)
            arch="x86_64"
            ;;
        aarch64 | arm64)
            arch="aarch64"
            ;;
        *)
            error "Unsupported architecture: $arch"
            ;;
    esac

    echo "${os}-${arch}"
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
check_prerequisites() {
    info "Checking prerequisites..."

    if ! command_exists curl && ! command_exists wget; then
        error "Neither curl nor wget found. Please install one of them."
    fi

    if ! command_exists tar; then
        error "tar is required but not found. Please install tar."
    fi
}

# Get the latest release version
get_latest_version() {
    info "Fetching latest release..."

    if command_exists curl; then
        curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
    elif command_exists wget; then
        wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
    fi
}

# Download and extract binary
download_binary() {
    local version="$1"
    local platform="$2"
    local download_url="https://github.com/${REPO}/releases/download/${version}/${BINARY_NAME}-${platform}.tar.gz"
    local tmp_dir

    tmp_dir="$(mktemp -d)"

    info "Downloading ${BINARY_NAME} ${version} for ${platform}..."

    if command_exists curl; then
        curl -sSL "$download_url" -o "${tmp_dir}/${BINARY_NAME}.tar.gz" || {
            warn "Pre-built binary not available for ${platform}"
            return 1
        }
    elif command_exists wget; then
        wget -qO "${tmp_dir}/${BINARY_NAME}.tar.gz" "$download_url" || {
            warn "Pre-built binary not available for ${platform}"
            return 1
        }
    fi

    info "Extracting binary..."
    tar -xzf "${tmp_dir}/${BINARY_NAME}.tar.gz" -C "${tmp_dir}"

    mkdir -p "${INSTALL_DIR}"
    mv "${tmp_dir}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    rm -rf "${tmp_dir}"

    return 0
}

# Build from source using cargo
build_from_source() {
    info "Pre-built binary not available. Building from source..."

    if ! command_exists cargo; then
        error "Cargo not found. Please install Rust: https://rustup.rs/"
    fi

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

    local platform
    platform="$(detect_platform)"
    info "Detected platform: ${platform}"

    local version
    version="$(get_latest_version)"

    if [ -z "$version" ]; then
        warn "Could not fetch latest version. Building from source..."
        build_from_source
    else
        info "Latest version: ${version}"

        if ! download_binary "$version" "$platform"; then
            build_from_source
        fi
    fi

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
    echo "Documentation: https://github.com/${REPO}#readme"
    echo ""
}

main
