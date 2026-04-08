# shoyu.nvim

[![Neovim plugin](https://img.shields.io/badge/neovim-plugin-57A143?logo=neovim)](https://neovim.io)
[![Top](https://img.shields.io/github/languages/top/aekasitt/shoyu.nvim)](https://github.com/aekasitt/shoyu.nvim)
[![Languages](https://img.shields.io/github/languages/count/aekasitt/shoyu.nvim)](https://github.com/aekasitt/shoyu.nvim)
[![Last commit](https://img.shields.io/github/last-commit/aekasitt/shoyu.nvim/master)](https://github.com/aekasitt/shoyu.nvim)

[![Shoyu banner](https://github.com/aekasitt/shoyu.nvim/blob/master/static/shoyu-banner.svg)](static/shoyu-banner.svg)

A powerful Neovim plugin that generates beautiful code snippet images for social media sharing.
Built with Rust for performance and powered by advanced syntax highlighting.

## Features

- 🎨 **8 Built-in Themes**: Dracula, Monokai, GitHub, Nord, Solarized Dark/Light, One Dark, and Gruvbox
- 🔥 **Syntax Highlighting**: Pattern-based highlighting for 20+ programming languages
- ⚡ **Fast**: Rust-powered shared library compilation
- 📱 **Social Media Ready**: High-resolution exports with 2x scaling
- 🎛️ **Highly Customizable**: Configurable fonts, padding, colors, and dimensions
- 📐 **Smart Sizing**: Auto-calculation of image dimensions based on content

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Neovim Lua    │    │   Rust FFI       │    │  Image Output   │
│   Interface     │◄──►│   Library        │───►│  (PNG/Base64)   │
│                 │    │                  │    │                 │
│ - Commands      │    │ - Rendering      │    │ - High-res      │
│ - Keymaps       │    │ - Highlighting   │    │ - Styled        │
│ - Config        │    │ - Themes         │    │ - Exportable    │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### Prerequisites

- **Neovim** (0.7+) with LuaJIT support
- **Rust** (for building the shared library)
- **Git**

### Using Lazy.nvim

```lua
{
  'aekasitt/shoyu.nvim',
  build = 'cargo build --release',
  config = function()
    require('shoyu').setup({
      theme = 'dracula',
      output_dir = vim.fn.expand('~/Pictures/shoyu'),
      font_size = 18,
      padding = 64,
      window_controls = true,
    })
  end,
  dependencies = {
    'nvim-lua/plenary.nvim',
  },
}
```

### Using Packer.nvim

```lua
use {
  'aekasitt/shoyu.nvim',
  run = 'cargo build --release',
  config = function()
    require('shoyu').setup()
  end
}
```

### Manual Installation

1. Clone the repository:
```bash
git clone https://github.com/aekasitt/shoyu.nvim.git
cd shoyu.nvim
```

2. Build the Rust library:
```bash
cargo build --release
```

3. Restart Neovim

## Usage

### Commands

- `:Shoyu [theme]` - Generate image from current buffer
- In `view` mode `:Shoyu [theme]` - Generate image from visual selection  
- `:ShoyuThemes` - List available themes

### Default Keymaps

- `<leader>sg` - Generate snippet image (normal mode: full buffer, visual mode: selection)
- `<leader>st` - Show available themes

### Configuration

```lua
require('shoyu').setup({
  -- Image dimensions
  width = 1200,          -- Image width in pixels
  height = nil,          -- Auto-calculate height
  padding = 64,          -- Padding around content
  export_size = 2.0,     -- Scale factor for high-res export
  
  -- Typography
  font_size = 18,        -- Font size
  font_family = "Fira Code",
  line_height = 1.5,     -- Line height multiplier
  
  -- Styling
  theme = "dracula",     -- Color theme
  window_controls = true, -- Show macOS-style window controls
  window_title = nil,    -- Optional window title
  line_numbers = false,  -- Show line numbers
  drop_shadow = true,    -- Add drop shadow effect
  border_radius = 8,     -- Corner radius
  background_color = "#1e1e1e", -- Custom background (overrides theme)
  
  -- Output
  output_dir = vim.fn.expand("~/Pictures/shoyu"),
  auto_open = true,      -- Automatically open generated images
})
```

## Themes

### Available Themes

1. **Dracula** - Dark theme with vibrant purple and green accents
2. **Monokai** - Classic dark theme with bright colors
3. **GitHub** - Clean light theme matching GitHub's interface
4. **Nord** - Arctic-inspired theme with cool colors
5. **Solarized Dark/Light** - Low-contrast, eye-friendly themes
6. **One Dark** - Atom's popular dark theme
7. **Gruvbox** - Retro groove color scheme

## API Reference

### Lua API

```lua
local shoyu = require('shoyu')

-- Generate image with custom options
shoyu.generate_image({
  theme = 'nord',
  font_size = 20,
  line_numbers = true,
  filename = 'my_snippet.png'
})

-- Generate from visual selection
shoyu.generate_from_selection({
  theme = 'github',
  padding = 32
})

-- Get available themes
local themes = shoyu.get_themes()
```

### Rust FFI Functions

The plugin exposes these C-compatible functions:

```rust
// Generate snippet image
generate_snippet_image(
  code: *const c_char,
  language: *const c_char, 
  theme: *const c_char,
  config_json: *const c_char
) -> *mut c_char

// Free allocated memory
free_string(s: *mut c_char)

// Get available themes
get_available_themes() -> *mut c_char

// Check language support
is_language_supported(language: *const c_char) -> c_int
```

## Contributions

### Building from Source

```bash
# Clone repository
git clone https://github.com/aekasitt/shoyu.nvim.git
cd shoyu.nvim
cargo build --release
```

### Project Structure

```
shoyu.nvim/
│
├── fonts/              # Collection of typeface files
│   ├── fallback.dat    # Fallback data served as basic font
│   └── *.ttf
│
├── lua/shoyu/
│   └── init.lua        # Neovim interface
│
├── plugin/
│   └── shoyu.lua       # Initiate Shoyu plugin
│
├── src/                # Rust backend to be compiled to shared library
│   ├── config.rs       # Configuration
│   ├── lib.rs          # Foreign function interface (FFI)
│   ├── renderer.rs     # Image generation
│   ├── syntax.rs       # Syntax highlighting
│   └── themes.rs       # Color themes
│
├── tests/              # Lua testsuites
│   └── *.lua
│
├── Cargo.lock          # Lockfile for Rust dependencies
├── Cargo.toml          # Rust build and project setup
└── build.lua           # Build shared library from source, autodetected by Lazy
```

### Adding New Themes

1. Add theme function in `src/themes.rs`:
```rust
fn my_theme() -> Theme {
  Theme {
    name: "My Theme".to_string(),
    background: ThemeColor::new("#282a36"),
    // ... other colors
  }
}
```

2. Register in `get_theme()` function
3. Add to `get_theme_names()` list

## Acknowledgments

* [Carbon](https://carbon.now.sh) 
  - [repository](https://github.com/ellisonleao/carbon-now.nvim)
* [EDMuzashi](https://www.f0nt.com/release/edmuzazhi) typeface
  by [อาทรเกติ์ แสงเพชร](https://www.facebook.com/ed.crub)
  nicknamed [ed_crub](https://www.f0nt.com/by/ed_crub)

## License

This project is licensed under the MIT License.
