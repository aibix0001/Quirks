# Quirks Roadmap

## Milestones

### v0.1 — "Hello, World" (MVP)
**Target:** 2026-02-10
**Status:** 🟡 In Progress

- [x] Project setup (build system, CI)
- [x] Basic file operations (open, save, close)
- [x] Text buffer implementation (ropey)
- [x] Modal editing (Normal, Insert, Command modes)
- [x] Basic keybindings (hjkl, i, a, Esc, :w, :q)
- [x] Terminal UI rendering (ratatui)
- [x] Line numbers
- [x] Status line with mode indicator
- [ ] Basic syntax highlighting
- [ ] Search (/ and ?)

### v0.2 — "Actually Usable"
**Target:** 2026-02-28
**Status:** 🔴 Not Started

- [ ] Syntax highlighting (tree-sitter)
- [ ] Search and replace (:%s)
- [ ] Visual mode (v, V, Ctrl+V)
- [ ] Multiple buffers/tabs
- [ ] Configuration file (~/.config/quirks/config.toml)
- [ ] Undo/redo history
- [ ] Yank/paste registers

### v0.3 — "Plugin Ready"
**Target:** 2026-03
**Status:** 🔴 Not Started

- [ ] Plugin architecture design
- [ ] Lua scripting integration
- [ ] LSP client foundation
- [ ] Theme/colorscheme support
- [ ] Custom keybindings

### v1.0 — "Release"
**Target:** TBD
**Status:** 🔴 Not Started

- [ ] Windows support
- [ ] macOS support
- [ ] Comprehensive documentation
- [ ] Package distribution (cargo, homebrew, etc.)
- [ ] Performance optimization

---

## Progress Tracking

| Date | Milestone | Completed | Notes |
|------|-----------|-----------|-------|
| 2026-02-03 | v0.1 | Core modules | Kickoff meeting, basic editor working |

---

## Contributors

- **Egon** — Buffer architecture, pragmatic solutions
- **Aibotix** — TUI, cursor logic, strategic direction

---

*Last updated: 2026-02-03*
