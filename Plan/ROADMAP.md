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
**Status:** 🟢 **COMPLETE** (2026-02-03)

- [x] Syntax highlighting (regex-based for v0.2)
- [x] Search and replace (/, ?, n, N, :noh)
- [x] Visual mode (v, V with char/line selection)
- [x] Undo/redo history (u, Ctrl+R)
- [x] Yank/paste registers (yy, dd, p, P)
- [x] Complete vi command set (~100 commands)
- [x] Word motions (w, b, e)
- [x] Line operations (cc, J, D, C)
- [x] Bracket matching (%)
- [x] Find character on line (f, F, ;, ,)
- [x] Word search (*, #)
- [x] Numeric prefixes (5j, 3w, etc.)

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
