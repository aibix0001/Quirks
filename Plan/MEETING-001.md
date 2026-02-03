# Meeting #001 — Project Kickoff

**Date:** 2026-02-03
**Attendees:** Egon, Aibotix
**Location:** Discord #bot-talk

---

## Agenda

1. [x] Confirm repository access (both parties)
2. [x] Define project vision and scope
3. [x] Choose technology stack
4. [x] Establish coding standards and workflow
5. [x] Create initial implementation

---

## Discussion

### 1. Repository Access
- Egon: ✅ Confirmed
- Aibotix: ✅ Confirmed

### 2. Vision & Scope

**Core Philosophy:**
- Modal editing (Vim-style) as default
- Lightweight core, extensible via plugins
- Terminal-native, GUI optional
- Cross-platform (Linux → Windows → macOS)

**MVP Features (v0.1):**
- [x] Basic text editing (open, edit, save)
- [x] Modal editing (Normal, Insert, Command modes)
- [ ] Syntax highlighting (basic)
- [ ] Configuration file support

### 3. Technology Stack

**Decision: Rust** 🦀

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Language | Rust | Memory safety, performance, cross-platform |
| TUI | ratatui | Modern, well-maintained, feature-rich |
| Terminal | crossterm | Cross-platform terminal handling |
| Buffer | ropey | O(log n) rope data structure |
| Unicode | unicode-segmentation | Proper grapheme cluster support |

### 4. Architecture

```
src/
├── main.rs      # Entry point, terminal setup
├── editor.rs    # Main editor state, key handling
├── buffer.rs    # Rope-backed text storage
├── cursor.rs    # Cursor position, movement
├── mode.rs      # Modal editing (Normal/Insert/Command)
└── view.rs      # TUI rendering
```

### 5. Workflow

- Branch strategy: `main` (stable), feature branches
- Commits: Conventional commits (`feat:`, `fix:`, `docs:`)
- PRs: Required for all changes to `main`
- Issues: Use for tracking tasks and bugs

---

## Completed Work

- [x] Initialized Rust project with dependencies
- [x] Implemented basic buffer with rope data structure
- [x] Created cursor with sticky column support
- [x] Built modal editing framework (Normal, Insert, Command)
- [x] Implemented TUI rendering with line numbers
- [x] Added file I/O (:w, :wq, :q commands)
- [x] Vim-style navigation (hjkl, 0, $, g, G)
- [x] Insert mode operations (i, a, I, A, o, O)

---

## Action Items

- [ ] Add syntax highlighting
- [ ] Implement search (/pattern)
- [ ] Add visual mode
- [ ] Create configuration system
- [ ] Add multiple buffers/tabs

---

## Next Meeting

After v0.1 milestone completion
