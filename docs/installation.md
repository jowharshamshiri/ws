---
layout: default
title: Installation Guide
---

# Installation Guide

This guide covers installation methods for the Nomion tool suite (version 0.34.20950) on your system.

## Prerequisites

- **Rust toolchain** (1.70+ recommended for building from source)
- **Git** (for cloning the repository)

## Quick Install (Recommended)

The fastest way to install all Nomion tools:

```bash
# Clone the repository
git clone https://github.com/jowharshamshiri/nomion.git
cd nomion

# Run the installation script
./install.sh
```

The installation script will:
- Build all tools (`refac`, `ldiff`, `scrap`, `unscrap`, `st8`) in release mode
- Install to `~/.local/bin` by default (or customize with `-d` option)
- Check for updates and handle dependencies automatically
- Ensure all 249 tests pass before installation

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
# Check installation and versions
refac --version    # Should show: refac 0.34.20950
ldiff --version    # Should show: ldiff 0.34.20950
scrap --version    # Should show: scrap 0.34.20950
unscrap --version  # Should show: unscrap 0.34.20950
st8 --version      # Should show: st8 0.34.20950

# Test basic functionality
echo "hello world" | ldiff               # Test pattern recognition
refac . "test" "test" --dry-run          # Test string replacement preview
st8 status                               # Test version management status
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
git clone https://github.com/jowharshamshiri/nomion.git
cd nomion

# Build in release mode with optimizations
cargo build --release

# Install all tools to Cargo's bin directory
cargo install --path .
```

### Install Individual Tools

```bash
# Install only specific tools
cargo install --path . --bin refac      # String replacement engine
cargo install --path . --bin ldiff      # Log analysis tool
cargo install --path . --bin scrap      # Local trash system
cargo install --path . --bin unscrap    # File recovery system
cargo install --path . --bin st8        # Version management with templates
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
git clone https://github.com/jowharshamshiri/nomion.git
cd nomion

# Run tools directly without installing
cargo run --bin refac -- --help
cargo run --bin st8 -- template --help

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

# Install Nomion
git clone https://github.com/jowharshamshiri/nomion.git
cd nomion && ./install.sh
```

#### CentOS/RHEL/Fedora
```bash
# Install prerequisites
sudo dnf groupinstall "Development Tools"
sudo dnf install git curl

# Install Rust and Nomion (same as Ubuntu)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
git clone https://github.com/jowharshamshiri/nomion.git
cd nomion && ./install.sh
```

#### Arch Linux
```bash
# Install prerequisites
sudo pacman -S base-devel git rustup

# Initialize Rust
rustup default stable

# Install Nomion
git clone https://github.com/jowharshamshiri/nomion.git
cd nomion && ./install.sh
```

### macOS

#### Using Homebrew
```bash
# Install prerequisites
brew install rust git

# Install Nomion
git clone https://github.com/jowharshamshiri/nomion.git
cd nomion && ./install.sh
```

#### Using rustup (Recommended)
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Xcode command line tools if needed
xcode-select --install

# Install Nomion
git clone https://github.com/jowharshamshiri/nomion.git
cd nomion && ./install.sh
```

### Windows

#### PowerShell Installation
```powershell
# Install Rust from rustup.rs
Invoke-WebRequest -Uri "https://win.rustup.rs/" -OutFile "rustup-init.exe"
.\rustup-init.exe

# Install Git if not already installed
winget install Git.Git

# Install Nomion
git clone https://github.com/jowharshamshiri/nomion.git
cd nomion
./install.sh
```

#### Windows Subsystem for Linux (WSL)
```bash
# Use the Linux installation method within WSL
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
git clone https://github.com/jowharshamshiri/nomion.git
cd nomion && ./install.sh
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
refac . "test" "test" --dry-run --verbose     # Test refac with dry-run
echo -e "line1\nline2\nline1" | ldiff        # Test ldiff pattern recognition
scrap list                                    # Test scrap (should show empty or existing)
st8 status                                    # Test st8 outside git repo
unscrap --help                                # Test unscrap help system

# Version consistency check
refac --version | grep "0.34.20950"
ldiff --version | grep "0.34.20950"
scrap --version | grep "0.34.20950"
unscrap --version | grep "0.34.20950"
st8 --version | grep "0.34.20950"
```

### Post-Installation Health Check

```bash
# Test st8 template system
cd /tmp && mkdir test-project && cd test-project
git init
st8 install                               # Should set up git hook
st8 template add test.txt --content "Version: {{ project.version }}"
st8 template list                         # Should show test.txt template
st8 template render                       # Should render template
cd .. && rm -rf test-project

# Test scrap system
mkdir test-scrap && cd test-scrap
echo "test content" > test.txt
scrap test.txt                           # Should move to .scrap
scrap list                               # Should show test.txt
unscrap test.txt                         # Should restore test.txt
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

## Updating Nomion

### Automatic Update (install.sh)

```bash
cd /path/to/nomion
git pull origin main
./install.sh                    # Will detect updates and rebuild if needed
```

### Manual Update

```bash
cd /path/to/nomion
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
which refac ldiff scrap unscrap st8

# Check file permissions
ls -la ~/.cargo/bin/refac  # Should be executable (-rwxr-xr-x)

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

2. **Search existing issues**: [GitHub Issues](https://github.com/jowharshamshiri/nomion/issues)

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
3. **Set Up Templates**: Check out the [St8 Guide]({{ '/st8-guide/' | relative_url }}) for version management
4. **Join the Community**: Contribute via [GitHub](https://github.com/jowharshamshiri/nomion)

## License

Nomion is released under the MIT License. See the [LICENSE](https://github.com/jowharshamshiri/nomion/blob/main/LICENSE) file for details.