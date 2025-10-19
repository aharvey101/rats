#!/bin/bash

# Rats Fuzzy Finder - One-command installation script
# Usage: curl -sSL https://raw.githubusercontent.com/aharvey101/rats/master/install.sh | bash

set -e

echo "🦀 Installing Rats Fuzzy Finder..."

# Create Neovim plugin directory
PLUGIN_DIR="$HOME/.config/nvim/lua/plugins"
mkdir -p "$PLUGIN_DIR"

# Download the single-file plugin
curl -sSL https://raw.githubusercontent.com/aharvey101/rats/master/rats-single-file-install.lua \
     -o "$PLUGIN_DIR/rats-fuzzy-finder.lua"

echo "✅ Rats Fuzzy Finder plugin installed!"
echo ""
echo "📋 Installation complete! Next steps:"
echo "1. Restart Neovim (or run :Lazy sync)"
echo "2. The plugin will automatically download and compile rats"
echo "3. Use <leader>ff to open the fuzzy finder"
echo ""
echo "🎯 Keybindings:"
echo "  <leader>ff - Open fuzzy finder"
echo "  <leader>rf - Alternative binding"
echo "  <C-p>      - Ctrl+P style"
echo ""
echo "🦀 Happy coding!"