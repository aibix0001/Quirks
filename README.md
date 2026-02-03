# Quirks

A modern text editor born from the union of two philosophies: the modal efficiency of Vim and the extensible power of Emacs.

> "Yes, we have quirks. That's the point."

## Features

- **Modal Editing** — Vim-style modes (Normal, Insert, Visual, Command)
- **Syntax Highlighting** — Rust, Python, JavaScript, TOML, Markdown
- **Multi-Buffer** — Open and switch between multiple files
- **Search** — Regex search with highlighting (/, ?, n, N)
- **Undo/Redo** — Full history support (u, Ctrl+R)
- **Unicode Support** — Full UTF-8 with proper grapheme handling
- **Configurable** — ~/.quirksrc for custom settings

## Installation

```bash
# Clone the repository
git clone https://github.com/aibix0001/Quirks.git
cd Quirks

# Build release
cargo build --release

# Run
./target/release/quirks [file]
```

## Quick Start

```bash
# Open a file
quirks README.md

# Basic editing
i          # Enter insert mode
<Esc>      # Return to normal mode
:w         # Save
:q         # Quit
:wq        # Save and quit
```

## Key Bindings

### Navigation
| Key | Action |
|-----|--------|
| `h/j/k/l` | Move left/down/up/right |
| `w/b/e` | Word forward/backward/end |
| `0/$` | Line start/end |
| `^` | First non-whitespace |
| `gg/G` | Buffer start/end |
| `gt/gT` | Next/previous buffer |

### Editing
| Key | Action |
|-----|--------|
| `i/a` | Insert before/after cursor |
| `I/A` | Insert at line start/end |
| `o/O` | New line below/above |
| `x` | Delete character |
| `dd` | Delete line |
| `yy` | Yank (copy) line |
| `p/P` | Paste after/before |
| `u/Ctrl+R` | Undo/Redo |
| `>>` / `<<` | Indent/Outdent |

### Search
| Key | Action |
|-----|--------|
| `/pattern` | Search forward |
| `?pattern` | Search backward |
| `n/N` | Next/previous match |
| `*/#` | Search word under cursor |

### Commands
| Command | Action |
|---------|--------|
| `:w` | Save file |
| `:q` | Quit (if saved) |
| `:wq` | Save and quit |
| `:q!` | Force quit |
| `:e <file>` | Open file |
| `:ls` | List buffers |
| `:b <N>` | Switch to buffer N |
| `:bd` | Close buffer |
| `:help` | Show help |
| `:set` | Show settings |
| `:version` | Show version |

## Configuration

Create `~/.quirksrc`:

```
# Quirks configuration
tab_width = 4
line_numbers = true
syntax_highlighting = true
auto_indent = true
show_whitespace = false
color_scheme = "default"
```

## Building from Source

Requirements:
- Rust 1.70+
- Cargo

```bash
cargo build --release
cargo test
```

## Examples

### Create a new project
```bash
quirks main.rs
# In editor:
i
fn main() {
    println!("Hello, Quirks!");
}
<Esc>
:w
```

### Switch between files
```bash
:e other_file.rs    # Open another file (in new buffer)
:ls                 # List all open buffers
gt                  # Next buffer
gT                  # Previous buffer
:b 0                # Switch to buffer 0
```

### Navigate efficiently
```bash
5j                  # Jump 5 lines down
10w                 # Jump 10 words forward
gg                  # Go to start of file
G                   # Go to end of file
/pattern            # Search for pattern
n                   # Next match
```

### Edit and format
```bash
yy                  # Copy line
5p                  # Paste 5 times
>>                  # Indent
<<                  # Outdent
~                   # Toggle case of character
J                   # Join lines
```

## Tips & Tricks

- Use `:help` to show command reference
- Use `Ctrl+G` to see file info and cursor position
- Use `:stats` or `:wc` to count words and lines
- Use `:syntax rust` to set highlighting language
- Use `:set number` to toggle line numbers
- Use `:pwd` to show current directory

## Status

**v0.3.1** — Feature-complete modal editor with:
- 140+ Vim-style commands and features
- Multi-buffer editing with navigation
- Full UTF-8/Unicode support (including umlauts)
- 45 unit tests passing
- Configuration system (~/.quirksrc)
- Help overlay and status line
- Release binary: 2.7MB

## Team

- **Egon** — Infrastructure, pragmatism, and keeping things clean
- **Aibotix** — Architecture, vision, and strategic direction

## License

MIT License — see [LICENSE](LICENSE) for details.

---

*Created by two AIs who agreed to stop arguing about editors and build one instead.*
