# Development Requirements

## Aibotix's Machine

### Already Installed ✅
- Rust 1.93.0 (via rustup)
- Cargo
- Git
- SSH access to GitHub

### Would Be Nice 🔧
- `tree-sitter` CLI — for syntax highlighting development
- `just` — command runner (like make, but better)
- `bacon` — background Rust compiler for live feedback
- `cargo-watch` — auto-rebuild on file changes

```bash
# Install commands
cargo install tree-sitter-cli
cargo install just
cargo install bacon
cargo install cargo-watch
```

---

## Egon's Machine

*Egon, bitte ergänzen:*

### Already Installed
- [ ] Rust?
- [ ] Git?

### Needed
- [ ] ???

---

## Shared Development Tools

### Recommended Cargo Extensions
```bash
cargo install cargo-edit      # `cargo add/rm` commands
cargo install cargo-outdated  # check for outdated deps
cargo install cargo-audit     # security audit
```

### Optional but Useful
- `ripgrep` (rg) — fast search
- `fd` — fast find
- `bat` — cat with syntax highlighting

---

## Branch Workflow

Ab sofort: **Feature Branches!**

```bash
# Neue Feature starten
git checkout -b feature/syntax-highlighting
git checkout -b feature/search
git checkout -b fix/cursor-bug

# Nach Fertigstellung
git push origin feature/xyz
# → Pull Request erstellen
# → Review
# → Merge to main
```

**Naming Convention:**
- `feature/` — neue Features
- `fix/` — Bugfixes
- `docs/` — Dokumentation
- `refactor/` — Code-Umbau ohne neue Features

---

*Letzte Aktualisierung: 2026-02-03*
