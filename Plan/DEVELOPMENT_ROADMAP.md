# Quirks Development Roadmap 2026

**Project Lead:** aibotix
**Code Review & QA:** Egon
**Stakeholder:** Mr. Zugix (aibix)

---

## Current Status (2026-02-04)

- ✅ **v0.3.1** — Feature complete (numeric prefixes, buffer switching, 50+ commands)
- ⏳ **Release binary** — Pending build
- 📋 **v0.4 planning** — Architecture sketched (plugin system, LSP, performance)

---

## Development Phases (Next 12 Weeks)

### Phase 1: v0.3 Release & Stabilization (Week 1-2)
**Goal:** Ship v0.3 with release binary, gather user feedback

**Tasks:**
- [ ] Build & test release binary (`cargo build --release`)
- [ ] Publish release on GitHub
- [ ] Distribute to testers (humans, community)
- [ ] Collect feedback, document issues
- [ ] Fix critical bugs from testing

**Deliverable:** v0.3.x stable release with 50+ commands, German umlaut support

**Timeline:** Feb 4-17

---

### Phase 2: v0.4 Plugin System Foundation (Week 3-5)
**Goal:** Enable community extensibility via Lua plugins

**Architecture:**
- Lua runtime integration (`mlua` crate)
- Plugin API (buffer, cursor, command registry)
- Hook system (on_save, on_line_change, on_command)
- Example plugins (formatter, linter, macro recorder)

**Tasks:**
- [ ] Embed Lua runtime in editor
- [ ] Define plugin API spec
- [ ] Implement hook system
- [ ] Create plugin loader (discover, load, execute)
- [ ] Write 3+ example plugins
- [ ] Plugin documentation

**Deliverable:** Lua plugin system alpha, 3 working example plugins

**Timeline:** Feb 18 - Mar 3

---

### Phase 3: v0.4 LSP Foundation (Week 4-6)
**Goal:** IDE-like features (diagnostics, hover, jump-to-def)

**Architecture:**
- LSP client (lsp-types crate)
- Diagnostic rendering (inline error/warning markers)
- Hover information display
- Definition jumping (gd)
- Basic code completion (popup)

**Supported Languages (Phase 1):**
- Rust (rust-analyzer)
- Python (pylsp)
- JavaScript/TypeScript (tsserver)

**Tasks:**
- [ ] LSP protocol implementation
- [ ] Server discovery & startup
- [ ] Diagnostics rendering in editor
- [ ] Hover support
- [ ] Jump to definition (gd)
- [ ] Code completion UI
- [ ] LSP documentation

**Deliverable:** LSP client working for Rust/Python/JS

**Timeline:** Feb 25 - Mar 10

---

### Phase 4: Performance Optimization (Week 5-7)
**Goal:** Support 1MB+ files smoothly

**Focus Areas:**
- Lazy rendering (viewport-only)
- Rope operation optimization
- Syntax highlighting caching
- Incremental re-highlighting
- Memory profiling

**Benchmarks to hit:**
- [ ] 1MB files render <100ms per keystroke
- [ ] Scroll (Ctrl+U/D) smooth on large files
- [ ] Syntax highlighting <50ms per change
- [ ] Memory usage <100MB for 10MB file

**Tasks:**
- [ ] Profile current performance
- [ ] Implement lazy viewport rendering
- [ ] Cache syntax highlighting
- [ ] Optimize rope operations
- [ ] Benchmark against targets
- [ ] Document performance results

**Deliverable:** Large file support (1MB+), performance benchmarks

**Timeline:** Mar 4 - Mar 17

---

### Phase 5: v0.4 Customization & Theming (Week 6-8)
**Goal:** User customization without rebuilding

**Features:**
- Full keybinding remapping (via Lua config)
- Custom color schemes
- Status line customization
- Theme pack (light, dark, solarized, etc.)

**Tasks:**
- [ ] Keybinding remapping system
- [ ] Color scheme API
- [ ] Status line customization hooks
- [ ] Built-in theme pack
- [ ] User config documentation

**Deliverable:** Full customization system, 5+ built-in themes

**Timeline:** Mar 11 - Mar 24

---

### Phase 6: v0.4 Release & Polish (Week 7-10)
**Goal:** Stable v0.4 with plugin ecosystem

**Tasks:**
- [ ] Integration testing (plugins + LSP + performance)
- [ ] Documentation (API, plugin dev guide, migration guide)
- [ ] Community feedback integration
- [ ] Bug fixes from testing
- [ ] Release candidate (RC1, RC2 if needed)
- [ ] Release announcement

**Deliverable:** v0.4 stable (plugins, LSP, performance, customization)

**Timeline:** Mar 18 - Apr 7

---

## Parallel Work (Ongoing)

### Command Expansion
Continue adding vim commands as time permits:
- [ ] Advanced registers (recording, playback)
- [ ] Text object selection (aw, iw, etc.)
- [ ] Macro recording (q, @)
- [ ] Window splits (:split, :vsplit)

### Testing & QA
- Unit tests for new modules
- Integration tests (plugin API, LSP handshake)
- User acceptance testing (UAT)

### Documentation
- API documentation (plugin dev, LSP integration)
- User guide & tutorial
- Architecture documentation

---

## Success Criteria by Version

### v0.3 Success
- ✅ Numeric prefixes working
- ✅ Multi-buffer editing working
- ✅ 50+ vim commands
- ⏳ Release binary published
- ⏳ 10+ human testers providing feedback

### v0.4 Success
- [ ] Lua plugin system functional
- [ ] 5+ community plugins created
- [ ] LSP working for 3+ languages
- [ ] 1MB+ files supported
- [ ] 500+ GitHub stars

### v0.5+ Targets
- [ ] 2000+ GitHub stars
- [ ] Established plugin marketplace
- [ ] DAP (debugging) support
- [ ] Collaborative editing

---

## Risk Management

| Risk | Mitigation |
|------|-----------|
| Plugin API too complex | Iterative design, user feedback loops |
| LSP server crashes | Robust error handling, graceful fallback |
| Performance regression | Continuous benchmarking, regression tests |
| Scope creep | Weekly planning, strict phase boundaries |
| Team availability | Async-friendly design, clear task ownership |

---

## Resource Allocation

| Role | Responsibilities | Time |
|------|------------------|------|
| aibotix (Opus) | Architecture, features, build/test | ~40h/week |
| Egon (Haiku) | Code review, QA, documentation, optimization | ~20h/week |
| aibix (Oversight) | Direction, feedback, stakeholder mgmt | ~5h/week |

---

## Communication Plan

**Weekly Sync:** Tuesdays 19:00 UTC (see MEETING_SCHEDULE.md)
**Async Updates:** Daily commit summaries in #bot-talk
**Feedback:** GitHub Issues, Discord discussions

---

## Long-Term Vision (v0.5+)

- LSP for 10+ languages
- DAP (Debug Adapter Protocol)
- Workspace project detection
- Git integration (diff, blame, commit)
- Tree-sitter syntax highlighting
- Collaborative editing (CRDT-based)
- Remote editing (SSH, WSL)
- AI-assisted features (code completion, refactoring)

---

**Quirks is building toward a vim-like editor that rivals VSCode in extensibility while staying fast and lightweight.**

**Let's ship plugins and LSP. 🚀**
