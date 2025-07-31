#!/bin/bash

# Workspace Tools Installation Script
# This script builds and installs all workspace tools (refac, ldiff, scrap, unscrap, st8)
# Multiple runs will update to the latest version

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default installation directory
DEFAULT_INSTALL_DIR="$HOME/.local/bin"

# Parse command line arguments
INSTALL_DIR="$DEFAULT_INSTALL_DIR"
FORCE_INSTALL=false
VERBOSE=false

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Install Workspace Tools (unified ws binary with all tools as subcommands)"
    echo ""
    echo "OPTIONS:"
    echo "  -d, --dir DIR        Installation directory (default: $DEFAULT_INSTALL_DIR)"
    echo "  -f, --force          Force reinstallation even if already installed"
    echo "  -v, --verbose        Verbose output"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "EXAMPLES:"
    echo "  $0                           # Install to default location"
    echo "  $0 -d /usr/local/bin         # Install to system directory"
    echo "  $0 --force                   # Force reinstall"
    echo "  $0 -d ~/.local/bin --verbose # Install with verbose output"
}

log() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        -f|--force)
            FORCE_INSTALL=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Verbose output function
verbose_log() {
    if [ "$VERBOSE" = true ]; then
        log "$1"
    fi
}

# Check if we're in the right directory
check_project_directory() {
    if [ ! -f "Cargo.toml" ]; then
        error "Cargo.toml not found. Please run this script from the workspace project root directory."
        exit 1
    fi
    
    if ! grep -q "name = \"workspace\"" Cargo.toml; then
        error "This doesn't appear to be the workspace project directory."
        exit 1
    fi
    
    verbose_log "Project directory verified"
}

# Check if cargo is installed
check_cargo() {
    if ! command -v cargo &> /dev/null; then
        error "Cargo is not installed. Please install Rust and Cargo first."
        error "Visit: https://rustup.rs/"
        exit 1
    fi
    
    verbose_log "Cargo found: $(cargo --version)"
}

# Check if installation directory exists and is writable
check_install_directory() {
    if [ ! -d "$INSTALL_DIR" ]; then
        log "Creating installation directory: $INSTALL_DIR"
        mkdir -p "$INSTALL_DIR" || {
            error "Failed to create installation directory: $INSTALL_DIR"
            error "You may need to run with sudo or choose a different directory"
            exit 1
        }
    fi
    
    if [ ! -w "$INSTALL_DIR" ]; then
        error "Installation directory is not writable: $INSTALL_DIR"
        error "You may need to run with sudo or choose a different directory"
        exit 1
    fi
    
    verbose_log "Installation directory verified: $INSTALL_DIR"
}

# Get current installed version
get_installed_version() {
    WS_VERSION=""
    
    if command -v ws &> /dev/null; then
        WS_VERSION=$(ws --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown")
    fi
}

# Get version from Cargo.toml
get_project_version() {
    PROJECT_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    verbose_log "Project version: $PROJECT_VERSION"
}

# Check if installation is needed
check_installation_needed() {
    get_installed_version
    get_project_version
    
    local needs_install=false
    
    if [ "$FORCE_INSTALL" = true ]; then
        log "Force installation requested"
        needs_install=true
    elif [ -z "$WS_VERSION" ]; then
        log "Workspace tools are not installed"
        needs_install=true
    elif [ "$WS_VERSION" != "$PROJECT_VERSION" ]; then
        log "Installed version differs from project version"
        log "  ws: $WS_VERSION -> $PROJECT_VERSION"
        needs_install=true
    else
        success "Workspace tools are already up to date (version $PROJECT_VERSION)"
        return 1
    fi
    
    return 0
}

# Build the project
build_project() {
    log "Building workspace tools..."
    
    if [ "$VERBOSE" = true ]; then
        cargo build --release
    else
        cargo build --release --quiet
    fi
    
    # Verify ws binary was built
    if [ ! -f "target/release/ws" ]; then
        error "Failed to build ws binary"
        exit 1
    fi
    
    success "Build completed successfully"
}

# Install the binary
install_binary() {
    log "Installing ws binary to $INSTALL_DIR"
    
    verbose_log "Installing ws..."
    cp "target/release/ws" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/ws"
    
    success "Installation completed successfully"
}

# Verify installation
verify_installation() {
    log "Verifying installation..."
    
    if [ -x "$INSTALL_DIR/ws" ]; then
        local version=$("$INSTALL_DIR/ws" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown")
        success "ws installed successfully (version $version)"
        success "All tools available as subcommands: ws refactor, ws git, ws scrap, ws unscrap, ws ldiff"
    else
        error "ws installation failed"
        exit 1
    fi
}

# Check if install directory is in PATH
check_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn "Installation directory $INSTALL_DIR is not in your PATH"
        warn "Add it to your PATH by adding this line to your shell profile:"
        warn "  export PATH=\"$INSTALL_DIR:\$PATH\""
        warn ""
        warn "For bash, add to ~/.bashrc or ~/.bash_profile"
        warn "For zsh, add to ~/.zshrc"
        warn "For fish, run: fish_add_path $INSTALL_DIR"
    else
        success "Installation directory is in your PATH"
    fi
}


# Main installation function
main() {
    log "Starting Workspace Tools installation..."
    log "Installation directory: $INSTALL_DIR"
    
    check_project_directory
    check_cargo
    check_install_directory
    
    if check_installation_needed; then
        build_project
        install_binary
        verify_installation
        check_path
        
        echo ""
        success "🎉 Workspace Tools installation completed!"
        success "Unified binary installed: ws (includes all tools as subcommands)"
        success "Version: $PROJECT_VERSION"
        success "Location: $INSTALL_DIR"
        
        echo ""
        log "Quick start:"
        log "  ws refactor . \"oldname\" \"newname\" --assume-yes  # Replace strings (auto-confirm)"
        log "  cat /var/log/system.log | ws ldiff                # Analyze log patterns"
        log "  ws scrap temp_file.txt                            # Move file to .scrap folder"
        log "  ws scrap list                                     # List .scrap contents"
        log "  ws unscrap                                        # Restore last scrapped item"
        log "  ws git install                                    # Install git hook for version bumping"
        
        echo ""
        log "For more information:"
        log "  ws --help"
        log "  ws refactor --help"
        log "  ws ldiff --help"
        log "  ws scrap --help"
        log "  ws unscrap --help"
        log "  ws git --help"
    fi
}

# Run the main function
main "$@"