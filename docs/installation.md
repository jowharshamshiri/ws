---
layout: default
title: Installation Guide
---

# Installation Guide

This guide covers installation methods for the Workspace tool suite.

## Prerequisites

- **Rust toolchain** (1.70+ recommended for building from source)
- **Git** (for cloning the repository and version management)
- **ripgrep** (`rg`) (required for wstemplate file discovery)

## Quick Install (Recommended)

```bash
git clone https://github.com/jowharshamshiri/wsb.git
cd workspace
./install.sh
```

The installation script will:
- Build the unified `wsb` binary in release mode (includes all tools as subcommands)
- Install to `~/.local/bin` by default (or customize with `-d` option)
- Check for updates and handle dependencies automatically

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
wsb --version
wsb --help
wsb refactor --help
wsb scrap --help
wsb git --help
wsb wstemplate --help
wsb version --help

# Test basic functionality
echo "hello world" | wsb ldiff               # Test pattern recognition
wsb refactor . "test" "test" --verbose       # Test string replacement preview
wsb git status                               # Test git integration status
```

### Uninstall

```bash
./uninstall.sh                         # Remove from default location
./uninstall.sh -d /usr/local/bin       # Remove from custom directory
```

## Manual Installation

### Build and Install

```bash
git clone https://github.com/jowharshamshiri/wsb.git
cd workspace
cargo build --release
cargo install --path .
```

### Custom Installation Location

```bash
cargo install --path . --root ~/.local
cargo install --path . --root /usr/local    # Requires sudo
```

## Development Installation

For development or testing the latest changes:

```bash
git clone https://github.com/jowharshamshiri/wsb.git
cd workspace

# Run tools directly without installing
cargo run -- refactor --help
cargo run -- wstemplate --help

# Build in debug mode for development
cargo build

# Run the test suite
cargo test
```

## Advanced Installation Options

### Performance Optimized Build

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
RUSTFLAGS="-C target-cpu=native" cargo install --path .
```

### Static Binary Build (Linux)

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

## Platform-Specific Installation

### Linux

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install build-essential git curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
git clone https://github.com/jowharshamshiri/wsb.git
cd workspace && ./install.sh
```

#### Arch Linux
```bash
sudo pacman -S base-devel git rustup
rustup default stable
git clone https://github.com/jowharshamshiri/wsb.git
cd workspace && ./install.sh
```

### macOS

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Xcode command line tools if needed
xcode-select --install

# Install ripgrep (required for wstemplate)
brew install ripgrep

# Install Workspace
git clone https://github.com/jowharshamshiri/wsb.git
cd workspace && ./install.sh
```

## Installation Verification

```bash
# Comprehensive functionality check
wsb refactor . "test" "test" --verbose        # Test refactor preview
echo -e "line1\nline2\nline1" | wsb ldiff     # Test ldiff pattern recognition
wsb scrap list                                # Test scrap
wsb git status                                # Test git integration
wsb unscrap --help                            # Test unscrap help
```

### Post-Installation Health Check

```bash
# Test template system
cd /tmp && mkdir test-project && cd test-project
git init
wsb git install
wsb template add test-template --template "Version: {{ project.version }}" --output test.txt
wsb template list
wsb template render
cd .. && rm -rf test-project

# Test scrap system
mkdir test-scrap && cd test-scrap
echo "test content" > test.txt
wsb scrap test.txt
wsb scrap list
wsb unscrap test.txt
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
```

## Updating Workspace

```bash
cd /path/to/workspace
git pull origin main
./install.sh        # Will detect updates and rebuild if needed
```

Or manually:

```bash
cd /path/to/workspace
git pull origin main
cargo build --release
cargo install --path . --force
```

## Troubleshooting

### Common Installation Issues

**"cargo: command not found"**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

**"Permission denied" during installation**
```bash
./install.sh -d ~/.local/bin
# Or use sudo for system installation
sudo ./install.sh -d /usr/local/bin
```

**"Binary not found after installation"**
```bash
echo $PATH | grep -E "(cargo/bin|local/bin)"
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Build Issues

**"xcrun: error: invalid active developer path" (macOS)**
```bash
xcode-select --install
```

**"failed to run custom build command" (Linux)**
```bash
sudo apt install build-essential  # Ubuntu/Debian
sudo dnf groupinstall "Development Tools"  # Fedora
```

### Runtime Issues

**"No such file or directory" when running tools**
```bash
which wsb
ls -la ~/.cargo/bin/wsb    # Should be executable
cargo install --path . --force
```

## Getting Help

If you encounter issues:

1. Check [Usage Guide]({{ '/usage/' | relative_url }}) and [API Reference]({{ '/api-reference/' | relative_url }})
2. Search [GitHub Issues](https://github.com/jowharshamshiri/wsb/issues)
3. Create a new issue with: OS/version, Rust version, error message, steps to reproduce

## Next Steps

1. **Quick Start**: [Getting Started Guide]({{ '/getting-started/' | relative_url }})
2. **Learn the Tools**: [Usage Guide]({{ '/usage/' | relative_url }})
3. **Version Management**: [St8 Guide]({{ '/st8-guide/' | relative_url }})
