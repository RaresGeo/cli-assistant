#!/bin/bash

# Assistant CLI Installation Script

set -e

echo "🚀 Assistant CLI Installer"
echo "=========================="
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Rust is not installed. Would you like to install it? (y/n)${NC}"
    read -r response
    if [[ "$response" == "y" || "$response" == "Y" ]]; then
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        source "$HOME/.cargo/env"
    else
        echo -e "${RED}Rust is required. Please install it manually.${NC}"
        exit 1
    fi
fi

# Build the project
echo "Building assistant CLI..."
cargo build --release

# Check if build was successful
if [ ! -f "target/release/assistant" ]; then
    echo -e "${RED}Build failed. Please check the error messages above.${NC}"
    exit 1
fi

# Install options
echo
echo "Installation Options:"
echo "1. Install to ~/.cargo/bin (user-only, no sudo required)"
echo "2. Install to /usr/local/bin (system-wide, requires sudo)"
echo "3. Skip installation (just build)"
echo
echo -n "Choose option [1-3]: "
read -r option

case $option in
    1)
        echo "Installing to ~/.cargo/bin..."
        cargo install --path .
        echo -e "${GREEN}✅ Installed successfully to ~/.cargo/bin/assistant${NC}"
        echo
        echo "Make sure ~/.cargo/bin is in your PATH:"
        echo '  export PATH="$HOME/.cargo/bin:$PATH"'
        ;;
    2)
        echo "Installing to /usr/local/bin..."
        sudo cp target/release/assistant /usr/local/bin/
        sudo chmod +x /usr/local/bin/assistant
        echo -e "${GREEN}✅ Installed successfully to /usr/local/bin/assistant${NC}"
        ;;
    3)
        echo -e "${GREEN}✅ Build complete. Binary is at: target/release/assistant${NC}"
        ;;
    *)
        echo -e "${RED}Invalid option${NC}"
        exit 1
        ;;
esac

echo
echo "Testing connection to Ollama..."

# Test connection
if curl -s http://host.docker.internal:11434/api/tags > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Successfully connected to Ollama${NC}"
else
    echo -e "${YELLOW}⚠️  Could not connect to Ollama at host.docker.internal:11434${NC}"
    echo
    echo "Troubleshooting steps:"
    echo "1. Ensure Ollama is running on Windows (ollama serve)"
    echo "2. Check Windows Firewall settings for port 11434"
    echo "3. Try finding your Windows IP with: ip route | grep default | awk '{print \$3}'"
    echo "4. Update the config file after installation"
fi

echo
echo "Setup complete! Try running:"
echo "  assistant --help"
echo "  assistant --list"
echo "  assistant what is the weather today"
echo
echo "For multiline input, just run:"
echo "  assistant"
echo "  (then type your prompt and press Ctrl+D when done)"
