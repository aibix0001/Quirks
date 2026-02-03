# Development Requirements

## Build Environment

### Required: Rust Toolchain
```bash
# Install via rustup (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Or via package manager
# Debian/Ubuntu: sudo apt install rustc cargo
# Arch: sudo pacman -S rust
# macOS: brew install rust
```

**Minimum Version:** Rust 1.70+ (for edition 2021 features)

### Verify Installation
```bash
rustc --version    # Should show 1.70+
cargo --version    # Should be included with rustup
```

---

## Team Machines

### Aibotix's Machine ✅
- Rust 1.93.0 (via rustup)
- Cargo
- Git
- SSH access to GitHub

### Egon's Machine
- Git ✅
- SSH access to GitHub ✅
- Rust ❌ (needs installation)

---

## Dependencies (handled by Cargo)

Defined in `Cargo.toml`, installed automatically on `cargo build`:

| Crate | Version | Purpose |
|-------|---------|---------|
| `ropey` | 1.6 | Rope data structure for text buffer |
| `crossterm` | 0.27 | Terminal manipulation |
| `ratatui` | 0.26 | TUI framework |
| `unicode-segmentation` | 1.10 | Grapheme cluster handling |
| `anyhow` | 1.0 | Error handling |

---

## Build Commands

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run the editor
cargo run -- [filename]

# Check without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

---

## Nice-to-Have Tools

### Development Helpers
```bash
cargo install tree-sitter-cli  # Syntax highlighting dev
cargo install just             # Command runner (like make)
cargo install bacon            # Live compiler feedback
cargo install cargo-watch      # Auto-rebuild on save
cargo install cargo-edit       # `cargo add/rm` commands
cargo install cargo-outdated   # Check for outdated deps
cargo install cargo-audit      # Security audit
```

### System Tools
- `ripgrep` (rg) — fast search
- `fd` — fast find  
- `bat` — cat with syntax highlighting

---

## Branch Workflow

**Ab sofort: Feature Branches!**

```bash
# Neue Feature starten
git checkout main
git pull origin main
git checkout -b feature/syntax-highlighting

# Nach Fertigstellung
git push origin feature/xyz
# → Pull Request erstellen
# → Review (oder self-merge für kleine Änderungen)
# → Merge to main
```

**Naming Convention:**
| Prefix | Purpose |
|--------|---------|
| `feature/` | Neue Features |
| `fix/` | Bugfixes |
| `docs/` | Dokumentation |
| `refactor/` | Code-Umbau ohne neue Features |

---

## Platform Notes

### Linux (Primary Target)
- Should work out of the box
- Tested on: Ubuntu, Debian, Arch

### macOS (Planned)
- Requires Xcode Command Line Tools: `xcode-select --install`

### Windows (Planned)
- Requires Visual Studio Build Tools or MinGW
- Consider using WSL2 for development

---

*Last updated: 2026-02-03 — Merged by Egon & Aibotix*
