# 🦀 Rats - Blazing Fast Fuzzy File Browser

A lightning-fast fuzzy file browser built with Rust and Ratatui, featuring seamless Neovim integration.

## Features

- ⚡ **Native Performance** - Built with Rust for maximum speed
- 🔍 **Smart Fuzzy Matching** - Intelligent scoring with consecutive character bonuses
- 📁 **Directory Navigation** - Expand folders and browse your entire project
- 🖥️ **Dual Mode Operation** - Interactive TUI or JSON output for integrations
- 🔌 **Neovim Integration** - Drop-in replacement with floating window UI
- 🛡️ **UTF-8 Safe** - Handles international filenames gracefully

## Installation

### Build from Source

```bash
git clone <your-repo>
cd rats
cargo build --release
cp target/release/rats ~/.local/bin/
```

### Verify Installation

```bash
rats --help
```

## Usage

### Interactive Mode (Default)

```bash
# Browse current directory
rats

# Browse specific directory
rats /path/to/directory

# Start with a search query
rats --query "main"
```

**Controls:**
- Type to filter files
- `↑/↓` or `j/k` - Navigate
- `Enter` - Open file/folder
- `Backspace` - Remove filter character
- `Ctrl+C` or `q` - Quit

### JSON Mode (For Integrations)

```bash
# Output search results as JSON
rats --json --query "cargo"
```

## Neovim Integration

### Setup

1. **Ensure rats is installed** in your `$PATH`

2. **Add the Lua module** to `~/.config/nvim/lua/config/fuzzy_finder.lua`:
   ```lua
   -- (The fuzzy_finder.lua content would go here)
   ```

3. **Add keybindings** to `~/.config/nvim/lua/config/keymaps.lua`:
   ```lua
   local fuzzy_finder = require('config.fuzzy_finder')
   vim.keymap.set('n', '<leader>ff', fuzzy_finder.find_files, { desc = 'Find files (rats fuzzy finder)' })
   ```

### Keybindings

- `<leader>ff` - Open fuzzy finder (primary)
- `<leader>rf` - Open fuzzy finder (alternative)
- `<C-p>` - Open fuzzy finder (Ctrl+P style)

### Usage in Neovim

1. Press `<leader>ff` to open the fuzzy finder
2. Type to search files in real-time
3. Use `↑/↓` or `Ctrl+j/k` to navigate
4. Press `Enter` to open the selected file
5. Press `Escape` to close

## Technical Details

### Fuzzy Matching Algorithm

- **Consecutive character bonus** - Rewards matching consecutive letters
- **Word boundary bonus** - Prioritizes matches at word starts
- **Case-insensitive matching** - Flexible search behavior
- **Separator awareness** - Understands file path structure

### Performance

- **Rust backend** - Native performance with minimal overhead
- **Efficient filtering** - Real-time search with large file sets
- **Memory efficient** - Low resource usage even on large projects

## Command Line Options

```bash
rats [OPTIONS] [DIRECTORY]

Options:
  --json              Output results as JSON (for integrations)
  --query <QUERY>     Start with search query
  <DIRECTORY>         Directory to browse (default: current)
```

## Development

### Prerequisites

- Rust 1.70+
- Cargo

### Building

```bash
cargo build          # Debug build
cargo build --release # Release build
```

### Testing

```bash
cargo test
```

## License

MIT License - See LICENSE file for details.

## Contributing

Contributions welcome! Please feel free to submit a Pull Request.