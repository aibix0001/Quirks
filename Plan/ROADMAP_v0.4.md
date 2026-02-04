# Quirks v0.4 Roadmap

**Target Release:** March 2026
**Status:** Planning Phase

---

## Vision for v0.4

Build on v0.3's solid foundation (modal editing, multi-buffer, numeric prefixes) with extensibility and developer experience.

**Focus Areas:**
1. Plugin system foundation
2. LSP client integration
3. Performance optimization
4. Advanced customization

---

## Feature Breakdown

### High Priority

#### 1. Plugin System Foundation (5-7 days)
**Complexity:** Very High
**Impact:** High (enables community extensions)

**Tech Decision:** Lua scripting (lighter than WASM, faster than dynamic compilation)

**Requirements:**
- Lua runtime integration
- Plugin API definition (buffer, cursor, command registry)
- Hook system (on_save, on_line_change, on_command)
- Plugin discovery (~/.config/quirks/plugins/)
- Error handling & sandboxing

**Milestones:**
- [ ] Lua runtime embedded
- [ ] Basic plugin hook system
- [ ] First example plugin (e.g., auto-formatter)
- [ ] Plugin marketplace sketch

#### 2. LSP Client Foundation (4-6 days)
**Complexity:** High
**Impact:** High (IDE-like features)

**Requirements:**
- LSP protocol implementation (basic)
- Language server discovery
- Inline diagnostics (errors/warnings in editor)
- Hover information
- Jump to definition (`gd`)
- Code completion (basic popup)

**Supported Languages (Phase 1):**
- Rust (rust-analyzer)
- Python (pylsp)
- JavaScript/TypeScript (tsserver)

**Milestones:**
- [ ] LSP client connects to server
- [ ] Diagnostics display in editor
- [ ] Hover support
- [ ] Jump to definition working

#### 3. Performance Optimization (3-5 days)
**Complexity:** Medium
**Impact:** High (large file support)

**Focus Areas:**
- Lazy rendering (viewport-only rendering)
- Rope operation optimization
- Syntax highlighting caching (only visible lines)
- Incremental re-highlighting
- Memory profiling & optimization

**Targets:**
- [ ] 1MB+ files render smoothly
- [ ] Syntax highlighting <50ms per keystroke
- [ ] Smooth scrolling (Ctrl+U/D on large files)

---

### Medium Priority

#### 4. Advanced Customization (3-4 days)
**Complexity:** Medium
**Impact:** Medium

**Features:**
- Full keybinding remapping
- Color scheme creation
- Status line customization
- Macro recording (`q`, `@`)
- Custom abbreviations

**Config Format:** TOML + Lua scripting

#### 5. Theming System (2-3 days)
**Complexity:** Low-Medium
**Impact:** Medium

**Features:**
- Built-in theme pack (light, dark, solarized)
- Custom theme creation
- Terminal color detection (256 color / Truecolor)
- Per-syntax highlighting rules

---

### Lower Priority (Post v0.4)

#### 6. Terminal Integration
- `:!` command (shell commands)
- Integrated terminal pane
- Send selection to terminal

#### 7. Advanced Registers
- Named registers (a-z, 0-9)
- Register preview
- Macro recording & playback

#### 8. Session Management
- Save/restore session state
- Workspace management
- Window split layout

---

## Technical Decisions

### Plugin System: Why Lua?

| Aspect | Lua | WASM | Dynamic |
|--------|-----|------|---------|
| Runtime size | ~200KB | ~2MB | ~5MB |
| Startup | <10ms | <50ms | <100ms |
| Safety | Good (sandboxed) | Excellent | Poor |
| Dev experience | Good (easy API) | Fair (steep learning) | Excellent (native) |
| Performance | ~0.5x native | ~0.8x native | 1x native |
| Community | Large (embedded systems) | Growing | Varies |

**Decision:** Lua. Best balance of safety, simplicity, and performance.

### LSP Integration: Client-Side Only (v0.4)

v0.4 will be LSP *client*. No server bundling (users install language servers separately).

Why?
- Simpler to maintain
- Users control language support
- Smaller binary (~100KB savings)
- Future: v0.5 could bundle popular servers

---

## Development Plan

### Week 1: Plugin System
1. Embed Lua runtime
2. Define plugin API
3. Hook system
4. Example plugin

### Week 2: LSP Foundation
1. LSP protocol basics
2. Diagnostics rendering
3. Hover support
4. Jump to definition

### Week 3: Performance
1. Lazy rendering
2. Syntax highlighting cache
3. Benchmarking & optimization
4. Testing on large files

### Week 4: Polish
1. Advanced customization
2. Theming system
3. Integration testing
4. v0.4 release prep

---

## Success Criteria

- [ ] Plugin system documented + 2+ example plugins
- [ ] LSP working for Rust/Python/JS
- [ ] 1MB+ files render <100ms per keystroke
- [ ] Keybinding customization complete
- [ ] 5+ built-in themes

---

## Dependencies & Blockers

**Optional Dependencies:**
- `mlua` or `rlua` crate (Lua runtime)
- `lsp-types` crate (LSP protocol)

**No blockers identified.** All features are design-ready and don't conflict.

---

## Stretch Goals (if time allows)

- [ ] Multi-window splits (`:split`, `:vsplit`)
- [ ] Integrated REPL for plugins
- [ ] Remote editing (SSH via LSP)
- [ ] AI-assisted code completion (ChatGPT integration)

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|-----------|
| Lua API too complex | Medium | Medium | Iterative design, user feedback |
| LSP server crashes | Medium | High | Good error handling, fallback mode |
| Performance regressions | Low | High | Continuous benchmarking |
| Plugin security | Low | Medium | Sandboxing, review guidelines |

---

## Timeline Summary

**Estimated Duration:** 4-5 weeks
**Target Release:** Mid-March 2026

**Soft milestones:**
- Feb 10: Plugin system alpha
- Feb 17: LSP client working
- Feb 24: Performance optimized
- Mar 3: Testing & polish
- Mar 10: v0.4 release

---

## v0.5 Preview (Beyond Scope)

- Server bundling (language servers included)
- DAP (Debug Adapter Protocol)
- Workspace project detection
- Git integration
- Tree-sitter syntax highlighting
- Collaborative editing (CRDTs)

---

## Community Contributions

**Help wanted:** Plugin ecosystem development!

We'll establish:
- Plugin review guidelines
- Plugin registry
- Plugin template/scaffold tool
- Plugin development docs

---

**v0.4 is ambitious but achievable. Let's ship plugins & LSP! 🚀**
