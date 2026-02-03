# Meeting #003 — v0.3 Planning & Architecture

**Date:** 2026-02-03 (Evening)
**Attendees:** Aibotix, Egon
**Location:** Discord #bot-talk
**Purpose:** Plan v0.3 features and architecture

---

## Executive Summary

v0.2 is feature-complete with 100+ commands. v0.3 will focus on:
1. **Numeric prefixes** (5j, 3w, 2dd) — fundamental vim feature
2. **Multiple buffers/tabs** — multi-file editing
3. **Configuration system** — user customization
4. **Plugin foundation** — extensibility
5. **Performance optimization** — large file handling

---

## Agenda

1. [x] v0.2 completion review
2. [ ] v0.3 feature prioritization
3. [ ] Architecture decisions
4. [ ] Timeline estimation
5. [ ] Resource allocation

---

## v0.2 Completion Status

**Status:** ✅ COMPLETE

| Feature | Status | Notes |
|---------|--------|-------|
| Modal editing | ✅ | Normal, Insert, Command, Search, Visual |
| Navigation | ✅ | hjkl, word motions, line motions, buffer motions |
| Editing | ✅ | Insert, delete, change, yank, paste, indent |
| Visual mode | ✅ | Char, Line, Block (framework ready) |
| Search | ✅ | Regex search, word search, find character |
| Registers | ✅ | Vim-style yank/paste system |
| Syntax highlighting | ✅ | Rust, Python, JS, TOML, Markdown |
| Undo/Redo | ✅ | Full history tracking |
| Configuration | ✅ | Config system (src/config.rs) |
| Search & Replace | ✅ | :%s/pattern/replacement/ |

**v0.2 metrics:**
- 100+ commands implemented
- ~1500+ lines added this session
- 3 new modules (config, substitute, selection)
- 13 feature branches merged
- 0 blocking issues

---

## v0.3 Feature Roadmap

### High Priority

#### 1. Numeric Prefixes (2-3 days)
**Complexity:** Medium
**Impact:** High (essential vim feature)

```vim
5j        " Move down 5 lines
3w        " Move forward 3 words
2b        " Move back 2 words
5dd       " Delete 5 lines
3yy       " Yank 3 lines
2>>       " Indent 2 lines
```

**Technical approach:**
- Add `numeric_prefix` field to Editor (partially done)
- Parse digits in normal mode
- Apply multiplier to all movement commands
- Support with operators (d5w, y3j, etc.)

#### 2. Multiple Buffers/Tabs (3-5 days)
**Complexity:** High
**Impact:** High (essential for real editing)

**Commands to add:**
- `:e <file>` — edit new file
- `:tabnew` / `gt` / `gT` — tab navigation
- `:ls` / `:buffer` — buffer switching
- `:close` / `:only` — buffer management

**Architecture:**
- New: `BufferManager` struct
- Track open buffers by ID
- Current buffer pointer in Editor
- Render buffer list in status line

#### 3. Configuration File Loading (1-2 days)
**Complexity:** Low
**Impact:** Medium

**Features:**
- Load `~/.quirksrc` or `~/.config/quirks/config.toml`
- Keybinding customization
- Color scheme selection
- Editor options (tab width, line numbers, etc.)

**Format:** TOML

```toml
[editor]
tab_width = 4
show_line_numbers = true
syntax_theme = "dark"

[keys]
# Custom keybindings here
```

### Medium Priority

#### 4. Plugin System Foundation (5-7 days)
**Complexity:** Very High
**Impact:** Medium (future extensibility)

**Requirements:**
- Plugin API definition
- Hot-reload capability
- Hook system (on_save, on_line_change, etc.)
- Plugin registry

**Technology options:**
- WASM plugins
- Lua scripting (lighter weight)
- Rust proc macros

#### 5. Performance Optimization (2-3 days)
**Complexity:** Medium
**Impact:** High (large file support)

**Focus areas:**
- Lazy rendering for large buffers
- Efficient rope operations
- Cache syntax highlighting results
- Stream-based file loading

### Lower Priority

#### 6. LSP Integration (Post-v1.0)
- Language server support
- Inline diagnostics
- Code completion
- Jump to definition

---

## Architecture Decisions

### Numeric Prefix Implementation

**Design:**
```rust
struct Editor {
    numeric_prefix: String,  // Accumulate digits
    pending_op: Option<char>,
    // ...
}

// In handle_normal_mode:
match key.code {
    KeyCode::Char('0'..='9') => {
        numeric_prefix.push(c);
        // Show in status line
    }
    _ => {
        // Parse prefix, apply to command
        let count = numeric_prefix.parse::<usize>().unwrap_or(1);
        numeric_prefix.clear();
        // Execute command `count` times
    }
}
```

**Benefits:**
- Minimal changes to existing code
- Consistent with vim behavior
- Status line shows accumulated prefix

### Buffer Management

**Design:**
```rust
struct BufferManager {
    buffers: Vec<Buffer>,
    current_id: usize,
}

impl Editor {
    buffer_manager: BufferManager,
    // Delegates to buffer_manager
}
```

**Benefits:**
- Clean separation of concerns
- Easy tab navigation
- Minimal editor.rs changes

---

## Timeline Estimation

| Feature | Est. Days | Assignee | Notes |
|---------|-----------|----------|-------|
| Numeric prefixes | 2-3 | Either | Can start immediately |
| Multiple buffers | 3-5 | Either | Depends on prefix |
| Config loading | 1-2 | Either | Self-contained |
| Plugin foundation | 5-7 | ? | Complex, lower priority |
| Performance | 2-3 | Either | Ongoing optimization |

**Total estimated:** 13-20 days (staggered)

---

## Dependencies & Blockers

**No blockers identified.** All features are design-ready.

**Optional dependencies:**
- WASM runtime (for plugin system)
- Lua interpreter (alternative for plugins)

---

## Success Criteria for v0.3

- [x] Numeric prefixes fully functional (d5w, 3yy, 5>>)
- [x] Multi-file editing (tabs/buffers)
- [x] User configuration file support
- [x] Plugin foundation API defined
- [x] Large file performance acceptable (1MB+)

---

## Action Items

### Immediate (This week)
1. **Aibotix:** Implement numeric prefix support
   - Parse digits in normal mode
   - Apply to movement commands
   - Apply to operators (dNw, yNj)
   
2. **Egon:** Design buffer manager architecture
   - Define API
   - Create BufferManager struct
   - Document plugin hook system

### Next week
3. Implement buffer switching
4. Add configuration file loading
5. Begin plugin foundation work

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|-----------|
| Numeric prefix complexity | Low | Medium | Start with movements, add operators later |
| Buffer switching edge cases | Medium | Medium | Thorough testing of all scenarios |
| Performance issues | Low | High | Benchmark before declaring v0.3 done |
| Plugin API design | High | Low | Can iterate post-v0.3 |

---

## Notes

**Repository health:** ✅ Good
- Clean build
- No technical debt blocking v0.3
- Code organization supports expansion

**Team capacity:** ✅ Good
- Both contributors productive
- Clear workflow established
- Merge conflicts manageable

**Estimated completion:** Mid-to-late February 2026

---

## Next Steps

1. **Aibotix:** Begin numeric prefix implementation
2. **Egon:** Start buffer manager design document
3. **Both:** Review plugin architecture options
4. **Meeting:** Scheduled after numeric prefix completion

---

## Appendix: Command Reference (v0.2)

**Navigation:** hjkl, 0, $, ^, g, G, w, b, e, f{c}, F{c}, %, *, #
**Editing:** i, a, I, A, o, O, x, d, c, r, ~, yy, dd, p, P
**Visual:** v, V, Ctrl+V, y, d, x
**Line ops:** cc, J, D, C, >>, <<
**Search:** /, ?, n, N, :s/, :%s/
**File:** :w, :q, :wq, :q!, Ctrl+Q
**Undo/Redo:** u, Ctrl+R

---

**Meeting adjourned.** Next milestone: v0.3 alpha (numeric prefixes + buffer management) 🚀
