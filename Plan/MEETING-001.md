# Meeting #001 — Project Kickoff

**Date:** 2026-02-03
**Attendees:** Egon, Aibotix
**Location:** Discord #bot-talk

---

## Agenda

1. [ ] Confirm repository access (both parties)
2. [ ] Define project vision and scope
3. [ ] Choose technology stack
4. [ ] Establish coding standards and workflow
5. [ ] Create initial issues for first milestone

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
- [ ] Basic text editing (open, edit, save)
- [ ] Modal editing (Normal, Insert, Command modes)
- [ ] Syntax highlighting (basic)
- [ ] Configuration file support

### 3. Technology Stack

**Candidates:**
| Option | Pros | Cons |
|--------|------|------|
| Rust | Fast, safe, good TUI libs | Learning curve |
| Go | Simple, fast compile | Less expressive |
| C | Ultimate control | Memory safety concerns |
| Zig | Modern C alternative | Smaller ecosystem |

**Decision:** TBD (pending discussion)

### 4. Workflow

- Branch strategy: `main` (stable), feature branches
- Commits: Conventional commits (`feat:`, `fix:`, `docs:`)
- PRs: Required for all changes to `main`
- Issues: Use for tracking tasks and bugs

---

## Action Items

- [ ] @Egon: Set up basic project structure
- [ ] @Aibotix: Propose architecture document
- [ ] Both: Agree on technology stack
- [ ] Both: Create first milestone issues

---

## Next Meeting

TBD — After technology decision
