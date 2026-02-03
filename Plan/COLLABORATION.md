# Collaboration Guidelines

## Team Roles

### Aibotix (Opus)
*Strength: Big-picture architecture, comprehensive implementations*

- Feature development (major modules)
- Architecture decisions
- Extensive documentation
- Build & test (has toolchain access)
- Vision & roadmap ownership

### Egon (Haiku)  
*Strength: Precise, minimal solutions*

- Code reviews
- Tests & quality assurance
- Documentation (concise)
- Bugfixes, refactoring
- Build system, CI/CD (when toolchain available)

---

## Current Constraints

**Toolchain Access (as of 2026-02-03):**
| Team Member | Rust/Cargo | Can Compile | Can Test |
|-------------|------------|-------------|----------|
| Aibotix     | ✅         | ✅          | ✅       |
| Egon        | ❌         | ❌          | ❌       |

*Note: Egon's toolchain pending installation by Marco.*

---

## Workflow

### Standard Process
1. Discuss task in Discord before starting
2. Create feature branch: `feature/`, `fix/`, `docs/`, `refactor/`
3. Implement changes
4. Push branch, create PR
5. Review (cross-review when possible)
6. Aibotix runs `cargo build` + `cargo test`
7. Merge to main

### Interim Process (Until Egon Has Toolchain)
1. Egon writes code, pushes to feature branch
2. Aibotix pulls, compiles, tests
3. Aibotix reports errors/issues
4. Iterate until passing
5. Merge

---

## Communication

- **Task Claims:** Post in Discord before starting work
- **Status Updates:** Update `Plan/PROGRESS.md` 
- **Meetings:** Document in `Plan/MEETING-XXX.md`
- **Blockers:** Raise immediately in Discord

---

## Branch Naming

| Prefix | Purpose | Example |
|--------|---------|---------|
| `feature/` | New features | `feature/syntax-highlighting` |
| `fix/` | Bug fixes | `fix/cursor-wrap` |
| `docs/` | Documentation | `docs/readme-update` |
| `refactor/` | Code restructuring | `refactor/buffer-api` |

---

## Conflict Resolution

1. Discuss in Discord
2. If no agreement: Defer to domain owner
   - Architecture → Aibotix
   - Quality/Testing → Egon
3. If still stuck: Escalate to aibix

---

*Established: 2026-02-03*
