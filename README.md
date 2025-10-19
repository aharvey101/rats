# 🦀 Rats - Blazing Fast Fuzzy File Browser

A lightning-fast fuzzy file browser built with Rust and Ratatui, featuring seamless Neovim integration.

## Features

- ⚡ **Native Performance** - Built with Rust for maximum speed
- 🔍 **Smart Fuzzy Matching** - Intelligent scoring with consecutive character bonuses
- 📁 **Directory Navigation** - Expand folders and browse your entire project
- 🎯 **Vim-like Navigation** - hjkl movement, normal/insert modes, gg/G jumps
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
- **Normal Mode (default):**
  - `j/k` or `↓/↑` - Navigate files
  - `h` - Go back to parent directory
  - `l` or `Enter` - Open file/enter directory
  - `gg` - Jump to first file
  - `G` - Jump to last file
  - `Ctrl+u` - Page up (half screen)
  - `Ctrl+d` - Page down (half screen)
  - `i`, `a`, `A` - Enter insert mode for typing
  - `q`, `Esc`, `Ctrl+C` - Quit
- **Insert Mode (for searching):**
  - Type to filter files in real-time
  - `Esc` - Return to normal mode
  - `Enter` - Open selected file/directory
  - `Ctrl+C` - Quit

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

1. **Press `<leader>ff`** to open the fuzzy finder (starts in normal mode)
2. **Navigate with vim keys:** `j/k` to move, `h` to go back, `l/Enter` to open
3. **Press `i`** to enter insert mode and type search query
4. **Press `Esc`** to return to normal mode for navigation
5. **Use `gg`/`G`** for quick jumps to top/bottom
6. **Use `Ctrl+u`/`Ctrl+d`** for page navigation
7. **Press `Enter`** to open the selected file

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