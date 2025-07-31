---
layout: default
title: Installation Guide
---

# Installation Guide

This guide covers installation methods for the Workspace tool suite (version 0.38.31859) on your system.

## Prerequisites

- **Rust toolchain** (1.70+ recommended for building from source)
- **Git** (for cloning the repository)

## Quick Install (Recommended)

The fastest way to install all Workspace tools:

```bash
# Clone the repository
git clone https://github.com/jowharshamshiri/workspace.git
cd workspace

# Run the installation script
./install.sh
```

The installation script will:
- Build the unified `ws` binary in release mode (includes all tools as subcommands)
- Install to `~/.local/bin` by default (or customize with `-d` option)
- Check for updates and handle dependencies automatically
- Ensure all tests pass before installation

### Installation Options

```bash
./install.sh --help                    # See all available options
./install.sh -d /usr/local/bin         # Install system-wide
./install.sh --force                   # Force reinstall even if up-to-date
./install.sh --verbose                 # Show detailed build output
./install.sh --check                   # Verify installation without installing
```

### Quick Verification

```bash
# Check installation and version
ws --version       # Should show: ws 0.38.31859

# Test subcommands
ws --help          # Show all available subcommands
ws refactor --help # Test refactor subcommand
ws ldiff --help    # Test ldiff subcommand
ws scrap --help    # Test scrap subcommand
ws unscrap --help  # Test unscrap subcommand
ws git --help      # Test git integration subcommand
ws template --help # Test template subcommand
ws update --help   # Test update subcommand

# Test basic functionality
echo "hello world" | ws ldiff               # Test pattern recognition
ws refactor . "test" "test" --verbose       # Test string replacement preview
ws git status                               # Test git integration status
```

### Uninstall

```bash
./uninstall.sh                         # Remove all tools from default location
./uninstall.sh -d /usr/local/bin       # Remove from custom directory
./uninstall.sh --verbose               # Show detailed removal process
```

## Manual Installation

For users who prefer manual control or want to install specific tools:

### Install All Tools

```bash
# Clone and build
git clone https://github.com/jowharshamshiri/workspace.git
cd workspace

# Build in release mode with optimizations
cargo build --release

# Install ws binary to Cargo's bin directory
cargo install --path .
```

### Alternative: Direct Cargo Install

```bash
# The ws binary includes all tools as subcommands
cargo install --path . --bin ws         # Unified binary with all tools
```

### Custom Installation Location

```bash
# Install to custom directory
cargo install --path . --root ~/.local

# Install to system directory (requires sudo)
cargo install --path . --root /usr/local
```

## Development Installation

For development or testing the latest changes:

```bash
# Clone the repository
git clone https://github.com/jowharshamshiri/workspace.git
cd workspace

# Run tools directly without installing
cargo run -- refactor --help
cargo run -- template --help

# Build in debug mode for development
cargo build

# Run the comprehensive test suite
cargo test                              # Run all 249 tests
cargo test --test st8_template_tests    # Run specific test suite
```

## Advanced Installation Options

### Performance Optimized Build

```bash
# Build with native CPU optimizations
RUSTFLAGS="-C target-cpu=native -C target-feature=+crt-static" \
  cargo build --release

# Install with optimizations
RUSTFLAGS="-C target-cpu=native" \
  cargo install --path .
```

### Static Binary Build (Linux)

```bash
# Install musl target for static linking
rustup target add x86_64-unknown-linux-musl

# Build static binary
cargo build --release --target x86_64-unknown-linux-musl

# Static binaries will be in target/x86_64-unknown-linux-musl/release/
```

### Cross-Platform Builds

```bash
# Windows (from Linux/macOS)
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu

# macOS (from Linux)
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

## Platform-Specific Installation

### Linux

#### Ubuntu/Debian
```bash
# Install prerequisites
sudo apt update
sudo apt install build-essential git curl

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Workspace
git clone https://github.com/jowharshamshiri/workspace.git
cd workspace && ./install.sh
```

#### CentOS/RHEL/Fedora
```bash
# Install prerequisites
sudo dnf groupinstall "Development Tools"
sudo dnf install git curl

# Install Rust and Workspace (same as Ubuntu)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
git clone https://github.com/jowharshamshiri/workspace.git
cd workspace && ./install.sh
```

#### Arch Linux
```bash
# Install prerequisites
sudo pacman -S base-devel git rustup

# Initialize Rust
rustup default stable

# Install Workspace
git clone https://github.com/jowharshamshiri/workspace.git
cd workspace && ./install.sh
```

### macOS

#### Using Homebrew
```bash
# Install prerequisites
brew install rust git

# Install Workspace
git clone https://github.com/jowharshamshiri/workspace.git
cd workspace && ./install.sh
```

#### Using rustup (Recommended)
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Xcode command line tools if needed
xcode-select --install

# Install Workspace
git clone https://github.com/jowharshamshiri/workspace.git
cd workspace && ./install.sh
```

### Windows

#### PowerShell Installation
```powershell
# Install Rust from rustup.rs
Invoke-WebRequest -Uri "https://win.rustup.rs/" -OutFile "rustup-init.exe"
.\rustup-init.exe

# Install Git if not already installed
winget install Git.Git

# Install Workspace
git clone https://github.com/jowharshamshiri/workspace.git
cd workspace
./install.sh
```

#### Windows Subsystem for Linux (WSL)
```bash
# Use the Linux installation method within WSL
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
git clone https://github.com/jowharshamshiri/workspace.git
cd workspace && ./install.sh
```

## Quality Assurance & Testing

### Pre-Installation Testing

```bash
# Run the complete test suite (249 tests)
cargo test

# Run specific test categories
cargo test --test integration_tests           # Cross-tool integration (18 tests)
cargo test --test refac_encoding_tests        # Encoding safety (7 tests)
cargo test --test st8_template_tests          # Template system (15 tests)
cargo test --test scrap_advanced_integration_tests  # Advanced workflows (21 tests)

# Run tests with verbose output
cargo test -- --nocapture

# Performance testing with optimizations
cargo test --release
```

### Installation Verification

```bash
# Comprehensive functionality check
ws refactor . "test" "test" --verbose        # Test refactor (shows preview automatically)
echo -e "line1\nline2\nline1" | ws ldiff     # Test ldiff pattern recognition
ws scrap list                                # Test scrap (should show empty or existing)
ws git status                                # Test git integration outside git repo
ws unscrap --help                            # Test unscrap help system

# Version consistency check
ws --version | grep "0.34.20950"
```

### Post-Installation Health Check

```bash
# Test template system
cd /tmp && mkdir test-project && cd test-project
git init
ws git install                           # Should set up git hook
ws template add test-template --template "Version: {{ project.version }}" --output test.txt
ws template list                         # Should show test-template
ws template render                       # Should render template
cd .. && rm -rf test-project

# Test scrap system
mkdir test-scrap && cd test-scrap
echo "test content" > test.txt
ws scrap test.txt                        # Should move to .scrap
ws scrap list                            # Should show test.txt
ws unscrap test.txt                      # Should restore test.txt
cd .. && rm -rf test-scrap
```

## Environment Configuration

### PATH Configuration

Add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.):

```bash
# If installed via cargo install
export PATH="$HOME/.cargo/bin:$PATH"

# If installed via install.sh to custom location
export PATH="$HOME/.local/bin:$PATH"

# If installed system-wide
export PATH="/usr/local/bin:$PATH"
```

### Shell Completion (Optional)

Generate shell completions for enhanced CLI experience:

```bash
# For bash
refac --completion bash > ~/.local/share/bash-completion/completions/refac
ldiff --completion bash > ~/.local/share/bash-completion/completions/ldiff
# (repeat for other tools)

# For zsh
refac --completion zsh > ~/.local/share/zsh/site-functions/_refac
# (repeat for other tools)

# For fish
refac --completion fish > ~/.config/fish/completions/refac.fish
# (repeat for other tools)
```

## Updating Workspace

### Automatic Update (install.sh)

```bash
cd /path/to/workspace
git pull origin main
./install.sh                    # Will detect updates and rebuild if needed
```

### Manual Update

```bash
cd /path/to/workspace
git pull origin main
cargo build --release           # Rebuild with latest changes
cargo install --path . --force  # Force reinstall
```

### Version Checking

```bash
# Check current version
refac --version

# Check for updates (manual)
git fetch && git log HEAD..origin/main --oneline
```

## Troubleshooting

### Common Installation Issues

**"cargo: command not found"**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

**"Permission denied" during installation**
```bash
# Install to user directory instead of system
./install.sh -d ~/.local/bin
# Or use sudo for system installation
sudo ./install.sh -d /usr/local/bin
```

**"Binary not found after installation"**
```bash
# Check PATH includes installation directory
echo $PATH | grep -E "(cargo/bin|local/bin)"

# Add to PATH if missing
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Build Issues

**"error: Microsoft Visual C++ 14.0 is required" (Windows)**
- Install Microsoft C++ Build Tools or Visual Studio with C++ support

**"xcrun: error: invalid active developer path" (macOS)**
```bash
xcode-select --install
```

**"failed to run custom build command" (Linux)**
```bash
# Install build essentials
sudo apt install build-essential  # Ubuntu/Debian
sudo dnf groupinstall "Development Tools"  # CentOS/RHEL/Fedora
```

### Runtime Issues

**"No such file or directory" when running tools**
```bash
# Verify installation
which ws

# Check file permissions
ls -la ~/.cargo/bin/ws  # Should be executable (-rwxr-xr-x)

# Re-install if corrupted
cargo install --path . --force
```

**Tests failing during installation**
```bash
# Run tests manually to see detailed errors
cargo test --verbose

# Run specific failing test
cargo test --test integration_tests -- --nocapture

# Skip tests and install anyway (not recommended)
cargo install --path . --force --no-test
```

## Getting Help

If you encounter issues:

1. **Check existing documentation**:
   - [Usage Guide]({{ '/usage/' | relative_url }}) - Comprehensive usage examples
   - [API Reference]({{ '/api-reference/' | relative_url }}) - Complete command documentation
   - [Examples]({{ '/examples/' | relative_url }}) - Real-world use cases

2. **Search existing issues**: [GitHub Issues](https://github.com/jowharshamshiri/workspace/issues)

3. **Create a new issue** with:
   - Operating system and version (`uname -a`)
   - Rust version (`rustc --version`)
   - Complete error message
   - Steps to reproduce the issue
   - Output of `cargo test` if build-related

## Next Steps

Once installed successfully:

1. **Quick Start**: Try the examples in the [Getting Started Guide]({{ '/getting-started/' | relative_url }})
2. **Learn the Tools**: Read the [Usage Guide]({{ '/usage/' | relative_url }}) for comprehensive examples
3. **Set Up Templates**: Check out the [St8 Guide]({{ '/st8-guide/' | relative_url }}) for git integration and templates
4. **Join the Community**: Contribute via [GitHub](https://github.com/jowharshamshiri/workspace)

## License

Workspace is released under the MIT License. See the [LICENSE](https://github.com/jowharshamshiri/workspace/blob/main/LICENSE) file for details.