# Quirks v0.3 — Release Notes

**Release Date:** 2026-02-04
**Status:** ✅ Ready for Testing

---

## What's New

### Major Features

#### 1. **Numeric Prefixes** (Phase 1)
Execute commands multiple times with counts:
- **Movements:** `5j` (down 5 lines), `3w` (forward 3 words), `2b` (back 2 words)
- **Operators:** `d5w` (delete 5 words), `y3j` (yank 3 lines), `2>>` (indent 2 lines)
- **Line ops:** `5dd` (delete 5 lines), `3yy` (yank 3 lines)

#### 2. **Buffer Switching & Tabs** (Phase 2)
Multi-file editing support:
- `:e <file>` — Edit new file
- `:tabnew` — Create new tab
- `:tabclose` / `:only` — Close/manage tabs
- `:ls` — List open buffers with actual buffer list
- `gt` / `gT` — Navigate tabs

#### 3. **Page Navigation**
- `Ctrl+U` — Page up (half)
- `Ctrl+D` — Page down (half)
- `Ctrl+B` — Page up (full)
- `Ctrl+F` — Page down (full)

#### 4. **Configuration System**
- `:set` — View/modify editor settings at runtime
- `~/.quirksrc` — User config file support (TOML)

#### 5. **Enhanced Help**
- `?` — Show help overlay with command categories
- Command reference organized by type (motions, operators, visual, search)

#### 6. **Quality Improvements**
- ✅ Fixed German umlauts (äöü, ß, etc.)
- ✅ 23 unit tests passing
- ✅ UTF-8 grapheme handling corrected
- ✅ Buffer manager with proper line tracking
- ✅ Syntax highlighting for 6+ languages

---

## Technical Details

### Build Info
```bash
cargo build --release
# Binary: target/release/quirks (~2-3 MB)
```

### Tested Features
- ✅ Character insertion with Unicode
- ✅ Numeric prefix parsing + application
- ✅ Buffer switching with proper cursor restoration
- ✅ Syntax highlighting with umlauts
- ✅ All vi commands from v0.2 + new features

### Known Limitations
- Plugin system: Not implemented yet (v0.4)
- LSP support: Foundation only (v0.4)
- Large file optimization: Planned (v0.4)

---

## Testing Checklist

**For Testers:**

### Basic Operations
- [ ] Open file: `./quirks README.md`
- [ ] Type text with German umlauts: `Größe, äöü, ß`
- [ ] Navigate with hjkl
- [ ] Insert/delete/change text

### Numeric Prefixes
- [ ] `5j` moves down 5 lines
- [ ] `3w` moves forward 3 words
- [ ] `d5w` deletes 5 words
- [ ] `2>>` indents 2 lines
- [ ] `5dd` deletes 5 lines

### Buffer Switching
- [ ] `:e othefile.txt` opens new file
- [ ] `:ls` shows buffer list
- [ ] `:tabnew` creates new tab
- [ ] Navigate between tabs (cursor positions persist)

### Page Navigation
- [ ] `Ctrl+F` page down
- [ ] `Ctrl+B` page up
- [ ] `Ctrl+U`/`Ctrl+D` half-page navigation

### Help & Config
- [ ] `?` shows help overlay
- [ ] `:set` displays current settings
- [ ] `q`/`Esc`/`Enter` closes help

### File Operations
- [ ] `:w` saves file
- [ ] `:wq` saves and quits
- [ ] `:q!` quits without saving

---

## v0.3.1 Commits (Last 10)

```
8227331 feat: add page navigation (Ctrl+U/D/B/F)
3ec86fb docs: update help overlay with more commands
92718bb feat: add Ctrl+G for file info
71e0293 feat: add :set commands for runtime configuration
d12c406 chore: update Cargo.toml to v0.3.1
a3ec947 test: add unit tests for Syntax Highlighter
13b5177 test: add unit tests for Search
1a0e14a test: add unit tests for Mode
43059eb feat: add :wa, :qa, :qa!, :wq! commands
e97f3e1 docs: comprehensive README update
```

---

## Download & Run

**Release Binary:**
```bash
# Option 1: Build locally
git clone https://github.com/aibix0001/Quirks.git
cd Quirks
cargo build --release
./target/release/quirks

# Option 2: Use pre-built binary (when available)
./quirks-v0.3-release
```

**Usage:**
```bash
quirks                    # Start editor
quirks README.md          # Open file
quirks file1.txt file2.txt # Open multiple files
```

---

## Feedback & Issues

Report bugs or feature requests on GitHub Issues: https://github.com/aibix0001/Quirks/issues

---

## v0.4 Roadmap (Coming Next)

- Plugin system foundation (Lua/WASM)
- LSP client (code completion, diagnostics)
- Performance optimization (large files)
- Theming system (user color schemes)
- Keybinding customization (full)

---

**v0.3 is feature-complete and ready for testing. Enjoy!** 🚀
